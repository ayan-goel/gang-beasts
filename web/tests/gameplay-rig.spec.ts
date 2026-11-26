import { expect, test } from '@playwright/test';
import { observeSkinDraws } from './support/skinDraws';

interface RigSample { tick: number; masses: number[]; pelvisX: number; finite: boolean }
declare global { interface Window { skylineRigSamples: (RigSample | undefined)[] } }

test('the playable WASM match draws both physical 40 kg rigs and steers the first pelvis', async ({page}) => {
  const drawn = await observeSkinDraws(page, 2);
  await page.goto('/play/');
  const start = page.getByRole('button',{name:'Start match'});
  await expect(start).toBeEnabled();
  await page.evaluate(async () => {
    const {LocalDuel} = await import(/* @vite-ignore */ String('/src/generated/wasm/brawler_wasm.js'));
    const read = LocalDuel.prototype.body_states;
    window.skylineRigSamples = [];
    LocalDuel.prototype.body_states = function(slot: number) {
      const pose: Float32Array = read.call(this,slot);
      window.skylineRigSamples[slot] = {
        tick: Number(this.tick()), pelvisX:pose[28]!, finite:pose.every(Number.isFinite),
        masses:Array.from(pose.filter((_,i) => i % 14 === 13)),
      };
      return pose;
    };
  });
  await start.click();
  await expect(page.getByLabel('Round phase')).toHaveText('Playing', {timeout:15000});
  await expect.poll(drawn).toBe(2);
  const first = await page.evaluate(() => window.skylineRigSamples[0]!);
  await page.locator('.scene-canvas canvas').focus();
  await page.keyboard.down('KeyD');
  try {
    await expect.poll(() => page.evaluate(() => window.skylineRigSamples[0]!.pelvisX), {timeout:12000})
      .toBeGreaterThan(first.pelvisX + 0.6);
  } finally { await page.keyboard.up('KeyD'); }
  const poses = await page.evaluate(() => window.skylineRigSamples);
  expect(poses.length).toBe(2);
  const expected = [4,10,6,1.5,1.5,1,1,0.5,0.5,3.5,3.5,2.5,2.5,1,1];
  for (const pose of poses) {
    expect(pose!.finite).toBe(true);
    expect(pose!.tick).toBeGreaterThan(first.tick);
    expect(pose!.masses).toHaveLength(15);
    pose!.masses.forEach((mass,i) => expect(mass).toBeCloseTo(expected[i]!,4));
    expect(pose!.masses.reduce((sum,value) => sum + value,0)).toBeCloseTo(40,4);
  }
});
