import { expect, test } from '@playwright/test';
import { execFileSync } from 'node:child_process';

test('locomotion WASM reproduces native movement, braking, impact and lifecycle traces', async ({ page }) => {
  await page.goto('/playground/'); await expect(page.getByRole('status')).toHaveText('Simulation ready');
  const frames = await page.evaluate(async () => {
    const { default: init, LocalRagdoll } = await import(/* @vite-ignore */ String('/src/generated/wasm/brawler_wasm.js'));
    await init();
    const rig = LocalRagdoll.locomotion(await (await fetch('/src/generated/physics.json')).text());
    const frame = () => ({ bodies: Array.from(rig.body_states()), joints: Array.from(rig.joint_states()), efforts: Array.from(rig.motor_efforts()), targets: Array.from(rig.motor_targets()), poses: Array.from(rig.pose_targets()), movement: Array.from(rig.movement_state()), motors: JSON.parse(rig.motor_configuration_json()) });
    const frames = [frame()];
    for (let step = 1; step <= 480; step++) {
      if (step === 61) rig.set_movement(1, 0, false);
      if ([121, 241, 421].includes(step)) rig.set_movement(0, 0, false);
      if (step === 181) rig.set_movement(0, -1, true);
      if ([271, 272].includes(step)) rig.queue_impulse(1, 10, 0, 0);
      if (step === 301) rig.set_enabled(false);
      if (step === 321) rig.set_enabled(true);
      if (step === 341) rig.reset();
      if (step === 361) rig.set_movement(0, -1, false);
      rig.step(); frames.push(frame());
    }
    rig.free(); return frames;
  });
  const native = JSON.parse(execFileSync('cargo', ['run', '--quiet', '--release', '--locked', '-p', 'brawler-engine', '--example', 'locomotion_trace'], {cwd: '..', encoding:'utf8', maxBuffer:16*1024*1024}));
  expect(frames).toHaveLength(native.length);
  let deviation = 0;
  for (let index = 0; index < frames.length; index++) {
    expect(frames[index]!.motors).toEqual(native[index].motors);
    for (const [key, size] of [['bodies',210],['joints',238],['efforts',42],['targets',42],['poses',56],['movement',11]] as const) {
      expect(frames[index]![key]).toHaveLength(size);
      for(let i=0;i<size;i++) deviation = Math.max(deviation, Math.abs(Number(frames[index]![key][i])-native[index][key][i]));
    }
  }
  expect(deviation).toBeLessThan(1e-5);
});
