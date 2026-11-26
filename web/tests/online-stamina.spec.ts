import { expect, test, type Page } from '@playwright/test';
import { decodeSnapshot } from '../src/net/snapshot';
import { approachOpponent, gripMatch } from './support/gripMatch';

function observeFatigue(page: Page) {
  const record = {minimumHeld: 100, releasedTired: false, refilled: false, reattached: false};
  page.on('websocket', socket => socket.on('framereceived', ({payload}) => {
    if (!Buffer.isBuffer(payload) || payload[0] !== 2) return;
    const owner = decodeSnapshot(payload).fighters[0]!;
    if (owner.grips[0]) {
      record.minimumHeld = Math.min(record.minimumHeld, owner.stamina);
      if (record.releasedTired) record.reattached = true;
    } else if (record.minimumHeld < 1) {
      record.releasedTired = true;
      if (owner.stamina === 100) record.refilled = true;
    }
  }));
  return record;
}

test('owner and opponent see a tiring grip, its release and stamina recovery', async ({page: host, browser}) => {
  await approachOpponent(host, 0);
  await gripMatch(host, browser, async (guest, start) => {
    const pages = [host, guest];
    const records = pages.map(observeFatigue);
    await start();
    await host.keyboard.down('KeyJ');
    await Promise.all(pages.map(async page => {
      await expect(page.getByLabel('Mint grips')).toContainText('Blue', {timeout: 25_000});
      await expect(page.getByRole('meter', {name: 'Mint stamina'})).toBeVisible();
      await expect(page.getByRole('meter', {name: 'Blue stamina'})).toHaveCount(0);
      await expect(page.getByText('Low stamina', {exact: true})).toBeVisible({timeout: 30_000});
      await expect(page.getByLabel('Mint grips')).toBeVisible();
      await expect(page.getByLabel('Mint grips')).toHaveCount(0, {timeout: 10_000});
      await expect(page.getByRole('meter', {name: 'Mint stamina'})).toBeVisible();
      await expect(page.getByRole('meter', {name: 'Mint stamina'})).toHaveCount(0, {timeout: 10_000});
    }));
    await expect.poll(() => records.every(r => r.refilled)).toBe(true);
    expect(records.every(r => !r.reattached)).toBe(true);
    await host.keyboard.up('KeyJ');
  });
});
