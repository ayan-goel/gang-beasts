import { test, expect } from '@playwright/test';
import { execFileSync } from 'node:child_process';

test('re-enabled fallen character uses bounded assistance and regains a physical stance', async ({page}) => {
  await page.goto('/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  const frames = await page.evaluate(async () => {
    const { default: init, LocalRagdoll } = await import(/* @vite-ignore */ String('/src/generated/wasm/brawler_wasm.js'));
    await init();
    const rig = LocalRagdoll.locomotion(await (await fetch('/src/generated/physics.json')).text());
    try {
      const frames = [];
      for (let tick = 1; tick <= 480; tick++) {
        if (tick === 61) { rig.set_enabled(false); rig.queue_impulse(1,10,0,0); }
        if (tick === 181) rig.set_enabled(true);
        rig.step();
        frames.push({bodies:Array.from<number>(rig.body_states()), recovery:Array.from<number>(rig.recovery_state())});
      }
      return frames;
    } finally { rig.free(); }
  });
  const native: typeof frames = JSON.parse(execFileSync('cargo', [
    'run','--quiet','--locked','--release','-p','brawler-engine','--example','recovery_trace',
  ], {cwd:'..',encoding:'utf8',maxBuffer:4*1024*1024}));
  expect(frames[179]!.bodies[1]).toBeLessThan(0.5);
  expect(frames.slice(180).some(frame => frame.recovery[0]! > 0)).toBe(true);
  expect(frames.slice(-60).every(frame => frame.bodies[1]! > 1.2 && frame.recovery[0] === 0)).toBe(true);
  let deviation = 0;
  expect(native).toHaveLength(480);
  for (const [tick, frame] of native.entries()) {
    for (const [field, len] of [['bodies',210],['recovery',3]] as const) {
      expect(frames[tick]![field]).toHaveLength(len);
      expect(frame[field]).toHaveLength(len);
      for (const [i, value] of frame[field].entries()) deviation = Math.max(deviation, Math.abs(value-frames[tick]![field][i]!));
    }
  }
  expect(deviation).toBeLessThan(1e-4);
});

test('playground shows physical get-up after a shove with control disabled', async ({page}) => {
  await page.goto('/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  await page.getByLabel('Test scene').selectOption('locomotion');
  const toggle = page.getByLabel('Enable character control');
  await page.getByLabel('Inspect body').selectOption('1');
  await toggle.uncheck();
  await page.getByRole('button',{name:'Shove right (10 N·s)'}).click();
  await page.getByLabel('Inspect body').selectOption('0');
  await page.getByRole('button',{name:'Resume',exact:true}).click();
  const height = async () => parseFloat(await page.locator('#height').innerText());
  await expect.poll(height,{timeout:10000}).toBeLessThan(0.5);
  await toggle.check();
  await expect(page.locator('#recovery-status')).toHaveText('Getting up',{timeout:10000});
  await expect.poll(height,{timeout:12000}).toBeGreaterThan(1.2);
  await expect(page.locator('#recovery-status')).toHaveText('Inactive');
});
