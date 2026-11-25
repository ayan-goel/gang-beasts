import { expect, test } from '@playwright/test';
import { execFileSync } from 'node:child_process';

test('stair playground exposes the physical staircase and locomotion controls', async ({ page }, testInfo) => {
  await page.goto('/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  await expect(page.locator('option[value="stairs"]')).toHaveText('Low stairs');
  await page.getByLabel('Test scene').selectOption('stairs');
  await expect(page.getByRole('heading', { name: 'Low stairs' })).toBeVisible();
  await expect(page.locator('canvas')).toHaveAttribute('aria-label', /stair/);
  await page.locator('.viewport').screenshot({ path: testInfo.outputPath('staircase.png') });
  await expect(page.locator('#tick')).toHaveText('0');
  await page.getByRole('button', { name: 'Resume', exact: true }).click();
  await page.getByRole('button', { name: 'Focus movement controls' }).click();
  await page.keyboard.down('w');
  await expect.poll(async () => Number(await page.locator('#position-z').textContent())).toBeLessThan(-1);
  await page.keyboard.up('w');
  await page.getByRole('button', { name: 'Pause', exact: true }).click();
  await page.getByRole('button', { name: 'Reset fixture' }).click();
  await expect(page.locator('#tick')).toHaveText('0');
  await expect(page.locator('#movement-command')).toHaveText('Idle');
});

test('stair terrain and every body frame survive reset and configuration replacement', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  const observed = await page.evaluate(async () => {
    const { default: init, LocalRagdoll } = await import(/* @vite-ignore */ String('/src/generated/wasm/brawler_wasm.js'));
    await init();
    const config = await (await fetch('/src/generated/physics.json')).text();
    const rig = LocalRagdoll.stairs(config);
    const replay = () => {
      const frames = [Array.from<number>(rig.body_states())];
      for (let tick = 1; tick <= 210; tick++) {
        if (tick === 61) rig.set_movement(0, -1, false);
        rig.step();
        frames.push(Array.from<number>(rig.body_states()));
      }
      return frames;
    };
    const terrain = rig.terrain_boxes();
    const frames = replay();
    rig.reset();
    const resetMatches = JSON.stringify(replay()) === JSON.stringify(frames);
    rig.configure(config);
    const configuredTerrain = Array.from<number>(rig.terrain_boxes());
    const configureMatches = JSON.stringify(replay()) === JSON.stringify(frames);
    rig.free();
    return { frames, resetMatches, configureMatches, configuredTerrain, terrain: Array.from<number>(terrain) };
  });
  expect(observed.resetMatches).toBe(true);
  expect(observed.configureMatches).toBe(true);
  expect(observed.configuredTerrain).toEqual(observed.terrain);
  expect(observed.terrain).toHaveLength(24);
  const tops = [0, 6, 12, 18].map(i => observed.terrain[i + 1]! + observed.terrain[i + 4]!);
  tops.forEach((height, i) => expect(height).toBeCloseTo(i * 0.1, 6));

  const native: number[][] = JSON.parse(execFileSync('cargo', [
    'run', '--quiet', '--locked', '--release', '-p', 'brawler-engine', '--example', 'stair_trace',
  ], { cwd: '..', encoding: 'utf8', maxBuffer: 4 * 1024 * 1024 }));
  expect(native).toHaveLength(211);
  expect(observed.frames).toHaveLength(native.length);
  let largestDifference = 0;
  for (const [tick, frame] of native.entries()) {
    expect(frame).toHaveLength(210);
    expect(observed.frames[tick]).toHaveLength(frame.length);
    for (const [i, value] of frame.entries()) {
      largestDifference = Math.max(largestDifference, Math.abs(value - observed.frames[tick]![i]!));
    }
  }
  expect(largestDifference).toBeLessThan(1e-5);
});
