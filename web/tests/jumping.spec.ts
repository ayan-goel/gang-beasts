import { test, expect } from '@playwright/test';
import { execFileSync } from 'node:child_process';

test('space and the Jump button lift the character and allow a supported landing', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  await page.getByLabel('Test scene').selectOption('locomotion');
  const jump = page.getByRole('button', { name: 'Jump', exact: true });
  await expect(jump).toBeDisabled();
  await page.getByRole('button', { name: 'Resume', exact: true }).click();
  const height = () => page.locator('#height').innerText().then(parseFloat);
  await expect.poll(async () => Number(await page.locator('#tick').textContent())).toBeGreaterThan(20);
  await page.keyboard.down('Space');
  await expect.poll(height, { intervals: [50] }).toBeGreaterThan(1.4);
  await expect.poll(height).toBeLessThan(0.95);
  await page.keyboard.up('Space');
  await jump.click();
  await expect.poll(height, { intervals: [50] }).toBeGreaterThan(1.4);
  await page.getByRole('button', { name: 'Pause', exact: true }).click();
  await expect(jump).toBeDisabled();
});

test('native and WASM agree on jumping, steering, holding, and rearming', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  const frames = await page.evaluate(async () => {
    const { default: init, LocalRagdoll } = await import(/* @vite-ignore */ String('/src/generated/wasm/brawler_wasm.js'));
    await init();
    const config = await (await fetch('/src/generated/physics.json')).text();
    const rig = LocalRagdoll.locomotion(config);
    try {
      for (const bad of [0, null, 'false']) {
        let rejected = false;
        try { rig.set_jump(bad); } catch { rejected = true; }
        if (!rejected) throw Error('jump held state must be boolean');
      }
      const frames: number[][] = [];
      for (let tick = 1; tick <= 220; tick++) {
        if (tick === 61 || tick === 160) rig.set_jump(true);
        if (tick === 70) rig.set_movement(1, 0, false);
        if (tick === 90) rig.set_movement(0, 0, false);
        if (tick === 155) rig.set_jump(false);
        rig.step();
        frames.push(Array.from<number>(rig.body_states()));
      }
      return frames;
    } finally { rig.free(); }
  });
  const native: number[][] = JSON.parse(execFileSync('cargo', [
    'run', '--quiet', '--locked', '--release', '-p', 'brawler-engine', '--example', 'jump_trace',
  ], {cwd:'..', encoding:'utf8', maxBuffer:4*1024*1024}));
  expect(native).toHaveLength(220);
  expect(frames).toHaveLength(native.length);
  let deviation = 0;
  for (const [tick, frame] of native.entries()) {
    expect(frame).toHaveLength(210);
    expect(frames[tick]).toHaveLength(frame.length);
    for (const [i, value] of frame.entries()) deviation = Math.max(deviation, Math.abs(value - frames[tick]![i]!));
  }
  expect(deviation).toBeLessThan(1e-5);
});
