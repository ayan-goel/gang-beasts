import { expect, test } from '@playwright/test';
import { execFileSync } from 'node:child_process';

test('standing WASM matches native control, push, disable, and reset traces', async ({ page }) => {
  await page.goto('/playground/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  const browser = await page.evaluate(async () => {
    const { default: init, LocalRagdoll } = await import(/* @vite-ignore */ String('/src/generated/wasm/brawler_wasm.js'));
    await init();
    const config = await (await fetch('/src/generated/physics.json')).text();
    const rig = LocalRagdoll.standing(config);
    const frame = () => ({ bodies: Array.from(rig.body_states() as Float32Array), joints: Array.from(rig.joint_states() as Float32Array), efforts: Array.from(rig.motor_efforts() as Float32Array), targets: Array.from(rig.motor_targets() as Float32Array), control: Array.from(rig.controller_state() as Float32Array) });
    const frames = [frame()];
    for (let step = 1; step <= 600; step++) {
      if (step >= 121 && step <= 123) rig.queue_impulse(1, 10, 0, 0);
      if (step === 360) rig.set_enabled(false);
      if (step === 420) rig.set_enabled(true);
      if (step === 480) rig.reset();
      rig.step(); frames.push(frame());
    }
    rig.reset();
    const motors = JSON.parse(rig.motor_configuration_json());
    const definition = JSON.parse(rig.definition_json());
    const caps = definition.joints.map((joint: { child: string }) => motors.joints.find((motor: {child: string}) => motor.child === joint.child).max_torque_nm) as number[];
    const standing = { minimumHead: Infinity, minimumPelvis: Infinity, maximumDrift: 0, maximumAnchor: 0, supportedTicks: 0, finite: true, boundedEffort: true, tick: '' };
    for (let step = 0; step < 3600; step++) {
      rig.step();
      const bodies = rig.body_states() as Float32Array;
      const joints = rig.joint_states() as Float32Array;
      const control = rig.controller_state() as Float32Array;
      const efforts = rig.motor_efforts() as Float32Array;
      standing.finite &&= bodies.length === 210 && joints.length === 238 && control.length === 6 && efforts.length === 42 && [bodies, joints, control, efforts].every(values => values.every(Number.isFinite));
      standing.minimumHead = Math.min(standing.minimumHead, bodies[1]!);
      standing.minimumPelvis = Math.min(standing.minimumPelvis, bodies[29]!);
      standing.maximumDrift = Math.max(standing.maximumDrift, Math.abs(bodies[0]!), Math.abs(bodies[2]!));
      standing.supportedTicks += Number(control[1] === 1 || control[2] === 1);
      standing.boundedEffort &&= Math.hypot(...control.subarray(3)) <= 30 + 1e-4;
      for (let joint = 0; joint < 14; joint++) {
        standing.maximumAnchor = Math.max(standing.maximumAnchor, joints[joint * 17 + 16]!);
        standing.boundedEffort &&= Math.hypot(...efforts.subarray(joint * 3, joint * 3 + 3)) <= caps[joint]! + 1e-4;
      }
    }
    standing.tick = String(rig.tick());
    rig.free();
    return { frames, standing, motors };
  });
  const native = JSON.parse(execFileSync('cargo', ['run', '--quiet', '--locked', '--release', '-p', 'brawler-engine', '--example', 'standing_trace'], { cwd: '..', encoding: 'utf8', maxBuffer: 16 * 1024 * 1024 })) as Record<'bodies' | 'joints' | 'efforts' | 'targets' | 'control', number[]>[];
  expect(browser.standing.tick).toBe('3600');
  expect(browser.standing.finite).toBe(true);
  expect(browser.standing.boundedEffort).toBe(true);
  expect(browser.standing.minimumHead).toBeGreaterThan(1.15);
  expect(browser.standing.minimumPelvis).toBeGreaterThan(0.65);
  expect(browser.standing.maximumDrift).toBeLessThan(0.2);
  expect(browser.standing.maximumAnchor).toBeLessThan(0.05);
  expect(browser.standing.supportedTicks).toBeGreaterThan(3500);
  expect(browser.motors.model).toBe('implicit_velocity_pd');
  expect(browser.frames).toHaveLength(native.length);
  let maximumDeviation = 0;
  for (let step = 0; step < native.length; step++) {
    for (const [field, size] of [['bodies', 210], ['joints', 238], ['efforts', 42], ['targets', 42], ['control', 6]] as const) {
      expect(browser.frames[step]![field]).toHaveLength(size);
      expect(native[step]![field]).toHaveLength(size);
      for (let index = 0; index < size; index++) maximumDeviation = Math.max(maximumDeviation, Math.abs(native[step]![field][index]! - browser.frames[step]![field][index]!));
    }
  }
  expect(maximumDeviation).toBeLessThan(1e-5);
});

test('standing scene exposes support, motor targets, and independent control enable', async ({ page }) => {
  await page.goto('/playground/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  await expect(page.locator('option[value="standing"]')).toHaveCount(1);
  await page.getByLabel('Test scene').selectOption('standing');
  await expect(page.getByLabel('Enable character control')).toBeChecked();
  await expect(page.locator('#tick')).toHaveText('0');
  await page.getByRole('button', { name: 'Single step' }).click();
  await page.getByRole('button', { name: 'Single step' }).click();
  await expect(page.locator('#foot-support')).toHaveText('Left + right');
  await page.getByLabel('Inspect body').selectOption('1');
  await expect(page.locator('#motor-target')).toContainText('Bind pose');
  await expect(page.locator('#motor-cap')).toHaveText('48.00 N·m');
  await page.getByLabel('Show joint frames').check();
  await expect(page.getByText('Pale parent axes show the bind-pose target')).toBeVisible();
  await page.getByLabel('Enable character control').uncheck();
  await expect(page.locator('#motor-effort')).toHaveText('0.00 N·m');
  await page.getByRole('button', { name: 'Reset fixture', exact: true }).click();
  await expect(page.getByLabel('Enable character control')).not.toBeChecked();
  await page.getByLabel('Enable character control').check();
  await page.getByRole('button', { name: 'Resume', exact: true }).click();
  await expect.poll(async () => Number(await page.locator('#tick').textContent())).toBeGreaterThan(120);
  await page.getByRole('button', { name: 'Pause', exact: true }).click();
  for (let hit = 0; hit < 3; hit++) {
    await page.getByRole('button', { name: 'Shove right (10 N·s)' }).click();
    await page.getByRole('button', { name: 'Single step' }).click();
  }
  await page.getByLabel('Inspect body').selectOption('0');
  await page.getByRole('button', { name: 'Resume', exact: true }).click();
  await expect.poll(async () => parseFloat(await page.locator('#height').textContent() ?? 'NaN'), { timeout: 10000 }).toBeLessThan(0.8);
  await page.getByRole('button', { name: 'Pause', exact: true }).click();
  await page.getByLabel('Test scene').selectOption('ragdoll');
  await expect(page.getByLabel('Enable character control')).toHaveCount(0);
  await expect(page.locator('canvas')).toHaveCount(1);
});
