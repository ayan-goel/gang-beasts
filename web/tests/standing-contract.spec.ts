import { expect, test } from '@playwright/test';

test('driven WASM validates control types, preserves mode, and owns diagnostic buffers', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  const result = await page.evaluate(async () => {
    const { default: init, LocalRagdoll } = await import(/* @vite-ignore */ String('/src/generated/wasm/brawler_wasm.js'));
    await init();
    const configuration = await (await fetch('/src/generated/physics.json')).text();
    const rig = LocalRagdoll.standing(configuration);
    const passive = new LocalRagdoll(configuration);
    const unsupported = [passive.motor_efforts().length, passive.motor_targets().length, passive.controller_state().length];
    let passiveRejected = false;
    try { passive.set_enabled(true); } catch { passiveRejected = true; }
    passive.free();
    const invalidBooleans: boolean[] = [];
    for (const value of [null, undefined, 0, 1, 'false', '', [], {}]) {
      try { rig.set_enabled(value); invalidBooleans.push(false); } catch { invalidBooleans.push(true); }
    }
    const remainedEnabled = rig.controller_state()[0];
    const invalidParts: boolean[] = [];
    for (const part of ['1', -1, 15, 1.5, NaN]) {
      try { rig.queue_impulse(part, 1, 0, 0); invalidParts.push(false); } catch { invalidParts.push(true); }
    }
    for (let step = 0; step < 40; step++) rig.step();
    const before = Array.from(rig.body_states() as Float32Array);
    let configureRejected = false;
    try { rig.configure('{}'); } catch { configureRejected = true; }
    const afterRejected = Array.from(rig.body_states() as Float32Array);
    const buffers = [rig.body_states(), rig.motor_efforts(), rig.motor_targets(), rig.controller_state()] as Float32Array[];
    const expected = buffers.map(buffer => Array.from(buffer));
    rig.set_enabled(false);
    rig.configure(JSON.stringify({ schema_version: 1, gravity_y: -6, solver_iterations: 12 }));
    const afterTuning = { tick: String(rig.tick()), control: Array.from(rig.controller_state() as Float32Array), model: JSON.parse(rig.motor_configuration_json()).model, configuration: JSON.parse(rig.configuration_json()) };
    rig.reset();
    const afterReset = Array.from(rig.controller_state() as Float32Array);
    rig.set_enabled(true);
    const afterEnableTargets = Array.from(rig.motor_targets() as Float32Array);
    rig.free();
    return { unsupported, passiveRejected, invalidBooleans, invalidParts, remainedEnabled, configureRejected, before, afterRejected, expected, retained: buffers.map(buffer => Array.from(buffer)), afterTuning, afterReset, afterEnableTargets };
  });
  expect(result.unsupported).toEqual([0, 0, 0]);
  expect(result.passiveRejected).toBe(true);
  expect(result.invalidBooleans).toEqual(Array(8).fill(true));
  expect(result.invalidParts).toEqual(Array(5).fill(true));
  expect(result.remainedEnabled).toBe(1);
  expect(result.configureRejected).toBe(true);
  expect(result.afterRejected).toEqual(result.before);
  expect(result.retained).toEqual(result.expected);
  expect(result.afterTuning.tick).toBe('0');
  expect(result.afterTuning.model).toBe('implicit_velocity_pd');
  expect(result.afterTuning.configuration.gravity_y).toBe(-6);
  expect(result.afterTuning.configuration.solver_iterations).toBe(12);
  expect(result.afterTuning.control).toEqual([0, 0, 0, 0, 0, 0]);
  expect(result.afterReset).toEqual([0, 0, 0, 0, 0, 0]);
  expect(result.afterEnableTargets).toEqual(Array(42).fill(0));
});
