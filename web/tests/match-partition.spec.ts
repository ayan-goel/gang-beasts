import { expect, test } from '@playwright/test';
import { outageRelay } from './support/outageRelay';
import { releaseRoom } from './support/releaseRoom';

for (const direction of ['down', 'up'] as const) {
  test('an open socket with silent ' + direction + 'link failure recovers the fighter', async ({page, browser, baseURL}) => {
    const relay = await outageRelay(baseURL!);
    const peer = await browser.newContext();
    const commands: number[] = [];
    const epochs: bigint[] = [];
    const assignments: number[] = [];
    page.on('websocket', socket => {
      socket.on('framesent', ({payload}) => {
        if (!Buffer.isBuffer(payload)) return;
        if (payload[0] === 16 || payload[0] === 32) commands.push(payload[0]);
        if (payload[0] === 24) epochs.push(payload.readBigUInt64LE(2));
      });
      socket.on('framereceived', ({payload}) => {
        if (Buffer.isBuffer(payload) && payload[0] === 23) assignments.push(payload[2]!);
      });
    });
    let invite: string | undefined;
    try {
      await page.addInitScript(port => {
        window.WebSocket = class extends WebSocket {
          constructor(url: string | URL, protocols?: string | string[]) {
            const path = new URL(url, location.href);
            if (path.pathname === '/ws') path.port = String(port);
            super(path, protocols);
          }
        };
      }, relay.port);
      await page.goto('/rooms/');
      await page.getByLabel('Player name').fill('Mint');
      await page.getByRole('button', {name:'Create room', exact:true}).click();
      invite = await page.getByLabel('Invite link').inputValue();
      const guest = await peer.newPage();
      await guest.goto(invite);
      await guest.getByLabel('Player name').fill('Gold');
      await guest.getByRole('button', {name:'Join room', exact:true}).click();
      for (const player of [guest, page]) await player.getByRole('button', {name:'Ready up'}).click();
      await page.getByRole('button', {name:'Start match'}).click();
      await expect(page.getByLabel('Round phase')).toHaveText('Playing', {timeout:15000});
      expect(commands).toEqual([16]);
      relay.cut(direction);
      await expect.poll(() => epochs.length, {timeout:18000}).toBe(2);
      expect(epochs[1]).toBe(epochs[0]);
      expect(assignments).toEqual([0,0]);
      expect(commands[0]).toBe(16);
      expect(commands.slice(1).every(command => command === 32)).toBe(true);
      expect(commands.length).toBeGreaterThan(1);
      for (const player of [page,guest]) await expect(player.getByLabel('Round phase')).toHaveText('Playing');
    } finally {
      try { await releaseRoom(invite, [page.context(),peer]); }
      finally { await peer.close(); await relay.close(); }
    }
  });
}
