import { expect, test } from '@playwright/test';
import { execFileSync } from 'node:child_process';

test('WASM ragdoll reproduces native body and joint-frame traces', async ({ page }) => {
  await page.goto('/playground/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  const browser = await page.evaluate(async () => {
    const { default: init, LocalRagdoll } = await import(/* @vite-ignore */ String('/src/generated/wasm/brawler_wasm.js'));
    await init();
    const config = await (await fetch('/src/generated/physics.json')).text();
    const rig = new LocalRagdoll(config);
    const frame = () => ({ bodies: Array.from(rig.body_states() as Float32Array), joints: Array.from(rig.joint_states() as Float32Array) });
    const frames = [frame()];
    for (let tick = 1; tick <= 240; tick++) {
      if (tick === 1) rig.queue_impulse(0, 3, 0, 1);
      if (tick === 90) rig.queue_impulse(7, 0, 2, 1);
      rig.step(); frames.push(frame());
    }
    const definition = JSON.parse(rig.definition_json());
    rig.free();
    return { frames, definition };
  });
  const native = JSON.parse(execFileSync('cargo', [
    'run', '--quiet', '--locked', '--release', '-p', 'brawler-engine', '--example', 'ragdoll_trace',
  ], { cwd: '..', encoding: 'utf8', maxBuffer: 8 * 1024 * 1024 })) as { bodies: number[]; joints: number[] }[];
  expect(browser.definition.bodies).toHaveLength(15);
  expect(browser.definition.joints).toHaveLength(14);
  expect(native).toHaveLength(browser.frames.length);
  let maximumDeviation = 0;
  for (let tick = 0; tick < native.length; tick++) {
    for (const field of ['bodies', 'joints'] as const) {
      expect(browser.frames[tick]![field]).toHaveLength(field === 'bodies' ? 210 : 238);
      expect(native[tick]![field]).toHaveLength(browser.frames[tick]![field].length);
      for (let index = 0; index < native[tick]![field].length; index++) {
        maximumDeviation = Math.max(maximumDeviation, Math.abs(native[tick]![field][index]! - browser.frames[tick]![field][index]!));
      }
    }
  }
  expect(maximumDeviation).toBeLessThan(1e-5);
});

test('WASM rig tuning and reset preserve ownership and reject invalid requests', async ({ page }) => {
  await page.goto('/playground/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  const result = await page.evaluate(async () => {
    const { default: init, LocalRagdoll } = await import(/* @vite-ignore */ String('/src/generated/wasm/brawler_wasm.js'));
    await init();
    const config = await (await fetch('/src/generated/physics.json')).text();
    const rig = new LocalRagdoll(config);
    const before = rig.body_states() as Float32Array;
    const expected = Array.from(before);
    const rejected: boolean[] = [];
    for (const part of [-1, 15, 256, 1.5, NaN, null, false, true, '', '7', [], {}, undefined]) {
      try { rig.queue_impulse(part, 0, 1, 0); rejected.push(false); } catch { rejected.push(true); }
      rig.reset();
    }
    rig.queue_impulse(0, 3, 0, 1); rig.step();
    const current = Array.from(rig.body_states() as Float32Array);
    let invalidThrew = false;
    try { rig.configure('{}'); } catch { invalidThrew = true; }
    const afterRejected = Array.from(rig.body_states() as Float32Array);
    rig.reset();
    const reset = Array.from(rig.body_states() as Float32Array);
    rig.step();
    const beforeTuningTick = String(rig.tick());
    rig.configure(JSON.stringify({schema_version: 1, gravity_y: -3, solver_iterations: 12}));
    const tick = String(rig.tick());
    const tuning = JSON.parse(rig.configuration_json());
    rig.free();
    return { rejected, expected, retained: Array.from(before), current, afterRejected, reset, tick, tuning, invalidThrew, beforeTuningTick };
  });
  expect(result.rejected).toEqual(Array(13).fill(true));
  expect(result.invalidThrew).toBe(true);
  expect(result.beforeTuningTick).toBe('1');
  expect(result.retained).toEqual(result.expected);
  expect(result.reset).toEqual(result.expected);
  expect(result.afterRejected).toEqual(result.current);
  expect(result.tick).toBe('0');
  expect(result.tuning.gravity_y).toBe(-3);
  expect(result.tuning.solver_iterations).toBe(12);
});
