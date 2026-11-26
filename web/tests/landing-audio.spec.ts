import { expect, test } from '@playwright/test';
import { gripMatch } from './support/gripMatch';
import { landingCount, observeWarningAudio } from './support/warningAudio';
import { decodeSnapshot } from '../src/net/snapshot';

test('a local jump sounds once on landing, including after a muted jump', async ({page}) => {
  await observeWarningAudio(page);
  await page.goto('/play/');
  await page.getByRole('button', {name:'Start match'}).click();
  await expect(page.getByLabel('Round phase')).toHaveText('Playing');
  const clock = page.locator('.match-round strong');
  await expect(clock).not.toHaveText(await clock.innerText());
  const mute = page.getByLabel('Mute game audio');
  await mute.click();
  await page.locator('canvas').focus();
  await page.keyboard.press('Space');
  const before = await clock.innerText();
  await expect(clock).not.toHaveText(before);
  await expect(clock).not.toHaveText(await clock.innerText());
  await mute.click();
  expect(await landingCount(page)).toBe(0);
  await page.locator('canvas').focus();
  await page.keyboard.down('Space');
  await expect.poll(() => landingCount(page)).toBe(1);
  await page.keyboard.up('Space');
  const after = await clock.innerText();
  await expect(clock).not.toHaveText(after);
  expect(await landingCount(page)).toBe(1);
});

test('both clients hear one landing for the same server-owned jump', async ({page:host,browser}) => {
  await gripMatch(host,browser,async (guest,start) => {
    const views = [host,guest];
    await Promise.all(views.map(observeWarningAudio));
    const events = new Set<number>();
    let tick = 0;
    host.on('websocket',socket => socket.on('framereceived', frame => {
      if (typeof frame.payload === 'string' || frame.payload[0] !== 2) return;
      const state = decodeSnapshot(frame.payload);
      tick = state.arenaTick;
      const event = state.fighters[0]?.landing;
      if (event) events.add(event.tick);
    }));
    await start();
    await expect.poll(() => tick).toBeGreaterThan(60);
    expect(events.size).toBe(0);
    await host.keyboard.press('Space');
    await expect.poll(() => Promise.all(views.map(landingCount))).toEqual([1,1]);
    expect(events.size).toBe(1);
    const landed = tick;
    await expect.poll(() => tick).toBeGreaterThan(landed + 30);
    expect(await Promise.all(views.map(landingCount))).toEqual([1,1]);
  });
});

test('a compact client without landing records can play in the same room', async ({page:host,browser}) => {
  await gripMatch(host,browser,async (guest,start) => {
    await guest.addInitScript(() => {
      const Socket = window.WebSocket;
      window.WebSocket = class extends Socket {
        constructor(address: string | URL, protocols?: string | string[]) {
          const url = new URL(address, location.href);
          url.searchParams.delete('landings');
          super(url, protocols);
        }
      };
    });
    let legacyTick = 0, landed = 0;
    guest.on('websocket', socket => socket.on('framereceived', ({payload}) => {
      if (typeof payload === 'string' || payload[0] !== 2) return;
      const block = 20 + 23 * payload.readUInt16LE(16);
      expect(payload[2]! & 4).toBe(4);
      expect(payload[block]).toBe(1);
      const state = decodeSnapshot(payload);
      expect(state.fighters.every(f => !f.landing)).toBe(true);
      legacyTick = state.arenaTick;
    }));
    host.on('websocket', socket => socket.on('framereceived', ({payload}) => {
      if (typeof payload !== 'string' && payload[0] === 2) {
        landed = decodeSnapshot(payload).fighters[0]?.landing?.tick ?? 0;
      }
    }));
    await start();
    await expect.poll(() => legacyTick).toBeGreaterThan(60);
    await host.keyboard.press('Space');
    await expect.poll(() => landed).toBeGreaterThan(60);
    await expect.poll(() => legacyTick).toBeGreaterThan(landed + 15);
    await expect(guest.getByLabel('Round phase')).toHaveText('Playing');
  });
});
