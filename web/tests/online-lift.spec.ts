import { expect, test } from '@playwright/test';
import { approachOpponent, gripMatch } from './support/gripMatch';
import { observeLift, type LiftSample } from './support/liftTrace';

if (process.platform === 'darwin') test.use({launchOptions: {args: ['--use-gl=angle', '--use-angle=metal']}});

const heldWithBoth = (s: LiftSample) => s.grips.every(id => id >= 17 && id <= 31);
const free = (s: LiftSample) => s.grips.every(id => id === 0);

test('both players see the opponent carry swing momentum through an airborne release', async ({page: host, browser}) => {
  const stopApproach = await approachOpponent(host, 0);
  await gripMatch(host, browser, async (guest, start) => {
    const views = [host, guest].map(observeLift);
    const owner = views[0]!;
    await start();
    await host.keyboard.down('KeyJ');
    await host.keyboard.down('KeyK');
    await expect.poll(() => heldWithBoth(owner.at(-1)!), {timeout: 12000}).toBe(true);
    stopApproach();
    await expect(host.getByText('Hold E to lift. Release your gripping hands to throw.')).toBeVisible();
    await expect(guest.getByText('Hold J or K to grab back with a free hand.')).toBeVisible();
    const base = owner.at(-1)!;
    await host.keyboard.down('KeyE');
    await expect.poll(() => {
      const latest = owner.at(-1)!;
      return heldWithBoth(latest) && latest.center[1] > base.center[1] + .035 && latest.clearance > .005;
    }, {intervals: [10], timeout: 7000}).toBe(true);
    await host.keyboard.up('KeyJ');
    await host.keyboard.up('KeyK');
    await host.keyboard.up('KeyE');
    for (const samples of views) {
      const released = () => samples.find(s => s.tick > base.tick && free(s));
      await expect.poll(() => {
        const first = released();
        return first && samples.at(-1)!.tick >= first.tick + 14;
      }).toBe(true);
      const first = released()!;
      const holding = samples.filter(s => s.tick >= base.tick && s.tick < first.tick && heldWithBoth(s));
      const before = holding.at(-2)!, last = holding.at(-1)!;
      const seconds = (last.tick-before.tick)/60;
      const vx = (last.center[0]-before.center[0])/seconds, vz = (last.center[2]-before.center[2])/seconds;
      const speed = Math.hypot(vx,vz);
      expect(speed).toBeGreaterThan(.08);
      expect(first.clearance).toBeGreaterThan(0);
      // Gravity changes vertical velocity throughout the arc. Compare the horizontal
      // speed along the swing between two free, airborne snapshots.
      const after = samples.find(s => s.tick > first.tick && s.tick <= first.tick + 8 && free(s) && s.clearance > 0)!;
      expect(after).toBeDefined();
      const dx = after.center[0]-first.center[0], dz = after.center[2]-first.center[2];
      const speedAlongSwing = (dx*vx+dz*vz)/speed * 60/(after.tick-first.tick);
      expect(speedAlongSwing).toBeGreaterThan(.5*speed);
    }
    await Promise.all([host,guest].map(page => expect(page.getByLabel('Mint grips')).toHaveCount(0)));
  });
});

test('the held fighter can grab back and keep the attachment after the lifter lets go', async ({page: host, browser}) => {
  const stopApproach = await approachOpponent(host, 0);
  await gripMatch(host, browser, async (guest, start) => {
    const views = [host, guest].map(observeLift);
    await start();
    await host.keyboard.down('KeyJ');
    await host.keyboard.down('KeyK');
    await expect.poll(() => heldWithBoth(views[0]!.at(-1)!), {timeout: 12000}).toBe(true);
    stopApproach();
    await guest.keyboard.down('KeyJ');
    await guest.keyboard.down('KeyK');
    const attachedBack = (s: LiftSample) => s.counter.some(id => id >= 1 && id <= 15);
    await expect.poll(() => views.every(samples => heldWithBoth(samples.at(-1)!) && attachedBack(samples.at(-1)!)), {timeout: 8000}).toBe(true);
    await host.keyboard.down('KeyE');
    await expect(guest.getByText('Releasing your hands leaves the other fighter’s grip attached.')).toBeVisible();
    await host.keyboard.up('KeyJ');
    await host.keyboard.up('KeyK');
    await host.keyboard.up('KeyE');
    await expect.poll(() => views.every(samples => free(samples.at(-1)!) && attachedBack(samples.at(-1)!))).toBe(true);
    await Promise.all([host,guest].map(page => expect(page.getByLabel('Blue grips')).toContainText('Mint')));
    await guest.keyboard.up('KeyJ');
    await guest.keyboard.up('KeyK');
    await Promise.all([host,guest].map(page => expect(page.getByLabel('Blue grips')).toHaveCount(0)));
  });
});
