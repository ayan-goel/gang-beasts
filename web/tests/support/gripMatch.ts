import { expect, type Browser, type Page } from '@playwright/test';
import { decodeSnapshot, type Snapshot } from '../../src/net/snapshot';
import { quantizeDirection } from '../../src/net/inputFrame';
import { releaseRoom } from './releaseRoom';

export async function gripMatch(host: Page, browser: Browser, run: (guest: Page, start: () => Promise<void>) => Promise<void>) {
  const profile = await browser.newContext();
  let invite = '';
  try {
    const guest = await profile.newPage();
    await run(guest, async () => {
      await host.goto('/rooms/');
      await host.getByLabel('Player name').fill('Mint');
      await host.getByRole('button',{name:'Create room',exact:true}).click();
      invite = await host.getByLabel('Invite link').inputValue();
      await guest.goto(invite);
      await guest.getByLabel('Player name').fill('Blue');
      await guest.getByRole('button',{name:'Join room',exact:true}).click();
      for (const p of [guest,host]) await p.getByRole('button',{name:'Ready up'}).click();
      await host.getByRole('button',{name:'Start match'}).click();
      await expect(host.getByLabel('Round phase')).toHaveText('Playing',{timeout:15_000});
      await host.bringToFront();
      await host.locator('canvas').focus();
    });
  } finally {
    try { await releaseRoom(invite,[host.context(),profile]); } finally { await profile.close(); }
  }
}

export async function approachOpponent(page: Page, hand: 0 | 1) {
  // Steer toward the opponent using the latest server positions. Hand buttons still come from the keyboard.
  await page.routeWebSocket(url => url.pathname === '/ws', client => {
    const server = client.connectToServer();
    let latest: Snapshot | undefined;
    server.onMessage(message => {
      if (Buffer.isBuffer(message) && message[0] === 2) latest = decodeSnapshot(message);
      client.send(message);
    });
    client.onMessage(message => {
      if (!latest || !Buffer.isBuffer(message) || message[0] !== 1) { server.send(message); return; }
      const outgoing = Buffer.from(message);
      const self = latest.bodies.get(3)!.position, target = latest.bodies.get(19)!.position;
      const dx = target[0]-self[0], dz = target[2]-self[2], length = Math.max(.001,Math.hypot(dx,dz));
      const [x,z] = latest.fighters[0]!.grips[hand] ? [0,0] : quantizeDirection(dx/length,dz/length);
      outgoing.writeInt8(x,10); outgoing.writeInt8(z,11);
      server.send(outgoing);
    });
  });
}
