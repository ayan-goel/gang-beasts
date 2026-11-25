import { execFileSync } from 'node:child_process';
import { expect, test } from '@playwright/test';

test('strike practice shows a target that swings after a punch and resets to rest', async ({ page }, testInfo) => {
  await page.goto('/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  await expect(page.locator('option[value="practice"]')).toHaveText('Strike practice');
  await page.getByLabel('Test scene').selectOption('practice');
  await expect(page.locator('canvas')).toHaveAttribute('aria-label', /suspended target/);
  await page.getByRole('button', { name: 'Resume', exact: true }).click();
  await expect.poll(async () => Number(await page.locator('#tick').textContent())).toBeGreaterThan(60);
  await expect(page.locator('#target-displacement')).toHaveText('0.00 m');
  await page.locator('canvas').focus();
  await page.keyboard.press('j');
  await expect.poll(async () => parseFloat(await page.locator('#target-displacement').innerText())).toBeGreaterThan(0.08);
  await page.getByRole('button', { name: 'Pause', exact: true }).click();
  await page.locator('.viewport').screenshot({ path: testInfo.outputPath('practice.png') });
  await page.getByRole('button', { name: 'Reset fixture' }).click();
  await expect(page.locator('#target-displacement')).toHaveText('0.00 m');
  await page.getByRole('button', { name: 'Apply tuning' }).click();
  await expect(page.getByRole('heading', { name: 'Strike practice' })).toBeVisible();
  await expect(page.locator('#target-displacement')).toHaveText('0.00 m');
});

test('both punches reproduce the native target trajectory across reset and tuning replacement', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  const observed = await page.evaluate(async () => {
    const { default: init, LocalRagdoll } = await import(/* @vite-ignore */ String('/src/generated/wasm/brawler_wasm.js'));
    await init();
    const config = await (await fetch('/src/generated/physics.json')).text();
    const rig = LocalRagdoll.practice(config);
    try {
      const play = (hand: number) => {
        const frames = [Array.from<number>(rig.practice_state())];
        for (let tick = 1; tick <= 150; tick++) {
          if (tick === 61) {
            rig.set_hand(hand, true);
            rig.set_hand(hand, false);
          }
          rig.step();
          frames.push(Array.from<number>(rig.practice_state()));
        }
        return frames;
      };
      const left = play(0);
      rig.reset();
      const right = play(1);
      rig.configure(config);
      return { traces: [left, right], reproduced: JSON.stringify(play(0)) === JSON.stringify(left) };
    } finally { rig.free(); }
  });
  const native: number[][][] = JSON.parse(execFileSync('cargo',
    ['run', '--quiet', '--release', '--locked', '-p', 'brawler-engine', '--example', 'practice_trace'],
    { cwd: '..', encoding: 'utf8' }));
  expect(observed.reproduced).toBe(true);
  let largestDifference = 0;
  for (const [side, trace] of native.entries()) {
    expect(trace).toHaveLength(151);
    for (const [tick, frame] of trace.entries()) {
      const actual = observed.traces[side]![tick]!;
      expect(actual).toHaveLength(13);
      for (const [i, value] of frame.entries()) {
        largestDifference = Math.max(largestDifference, Math.abs(actual[i]! - value));
      }
    }
  }
  expect(largestDifference).toBeLessThan(1e-5);
});
