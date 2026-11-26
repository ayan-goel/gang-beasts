import { expect, test, type Page } from '@playwright/test';
import { decodeSnapshot } from '../src/net/snapshot';
import { gripMatch } from './support/gripMatch';
import { footstepCount, observeWarningAudio } from './support/warningAudio';

function observeSteps(page: Page) {
  const wire = {events:new Set<number>(), idle:new Set<number>(), version:0, tick:0};
  page.on('websocket', socket => socket.on('framereceived', ({payload}) => {
    if (typeof payload === 'string' || payload[0] !== 2) return;
    const state = decodeSnapshot(payload);
    wire.version = payload[20 + 23 * payload.readUInt16LE(16)]!;
    wire.tick = state.arenaTick;
    const [moving, idle] = state.fighters;
    if (moving?.footstep) wire.events.add(moving.footstep);
    if (idle?.footstep) wire.idle.add(idle.footstep);
  }));
  return wire;
}

test('local travel sounds on contact and standing still stays quiet', async ({page}) => {
  await observeWarningAudio(page);
  await page.goto('/play/');
  await page.getByRole('button',{name:'Start match'}).click();
  await expect(page.getByLabel('Round phase')).toHaveText('Playing');
  const clock = page.locator('.match-round strong');
  await expect(clock).not.toHaveText(await clock.innerText());
  expect(await footstepCount(page)).toBe(0);
  await page.locator('canvas').focus();
  await page.keyboard.down('d');
  await expect.poll(() => footstepCount(page)).toBeGreaterThanOrEqual(2);
  await page.keyboard.up('d');
  await expect(clock).not.toHaveText(await clock.innerText());
  const stopped = await footstepCount(page);
  await expect(clock).not.toHaveText(await clock.innerText());
  expect(await footstepCount(page)).toBe(stopped);
  await page.getByLabel('Mute game audio').click();
  await page.locator('canvas').focus();
  await page.keyboard.down('a');
  await expect(clock).not.toHaveText(await clock.innerText());
  await expect(clock).not.toHaveText(await clock.innerText());
  await page.keyboard.up('a');
  expect(await footstepCount(page)).toBe(stopped);
});

test('both players receive and hear steps from the moving fighter', async ({page:host,browser}) => {
  await gripMatch(host,browser,async (guest,start) => {
    const pages = [host,guest];
    await Promise.all(pages.map(observeWarningAudio));
    const wires = pages.map(observeSteps);
    await start();
    await expect.poll(() => Math.min(...wires.map(w => w.tick))).toBeGreaterThan(60);
    expect(await Promise.all(pages.map(footstepCount))).toEqual([0,0]);
    await host.keyboard.down('d');
    await expect.poll(async () => Math.min(...await Promise.all(pages.map(footstepCount)))).toBeGreaterThanOrEqual(2);
    await host.keyboard.up('d');
    const end = Math.max(...wires.map(w => w.tick)) + 20;
    await expect.poll(() => Math.min(...wires.map(w => w.tick))).toBeGreaterThan(end);
    expect([...wires[0]!.events]).toEqual([...wires[1]!.events]);
    for (const [i,wire] of wires.entries()) {
      expect(wire.version).toBe(3);
      expect(wire.idle.size).toBe(0);
      expect(await footstepCount(pages[i]!)).toBeLessThanOrEqual(wire.events.size);
    }
  });
});

test('landing-only and footstep-capable clients share a room', async ({page:host,browser}) => {
  await gripMatch(host,browser,async (guest,start) => {
    await guest.addInitScript(() => {
      const Socket = window.WebSocket;
      window.WebSocket = class extends Socket {
        constructor(address: string | URL, protocols?: string | string[]) {
          const url = new URL(address,location.href);
          url.searchParams.delete('footsteps');
          super(url,protocols);
        }
      };
    });
    const modern = observeSteps(host), legacy = observeSteps(guest);
    await start();
    await host.keyboard.down('d');
    await expect.poll(() => modern.events.size).toBeGreaterThanOrEqual(2);
    await host.keyboard.up('d');
    const end = modern.tick + 15;
    await expect.poll(() => legacy.tick).toBeGreaterThan(end);
    expect(legacy.version).toBe(2);
    expect(legacy.events.size).toBe(0);
    await expect(guest.getByLabel('Round phase')).toHaveText('Playing');
  });
});
