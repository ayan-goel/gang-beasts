import { expect, test } from '@playwright/test';
import { execFileSync } from 'node:child_process';

type Frame = { bodies: number[]; movement: number[]; moment: number[] };
type Case = { direction: [number, number]; running: boolean; duration: number; reverse: number; frames: Frame[] };

test('stopping at varied phases preserves native posture, settling and moment limits in WASM', async ({ page }) => {
  const native: Case[] = JSON.parse(execFileSync('cargo', ['run', '--quiet', '--locked', '--release', '-p', 'brawler-engine', '--example', 'stopping_trace'], {
    cwd: '..', encoding: 'utf8', maxBuffer: 32 * 1024 * 1024,
  }));
  await page.goto('/playground/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  const traces: Frame[][] = await page.evaluate(async (cases) => {
    const { default: init, LocalRagdoll } = await import(/* @vite-ignore */ String('/src/generated/wasm/brawler_wasm.js'));
    await init();
    const config = await (await fetch('/src/generated/physics.json')).text();
    return cases.map(({ direction: [x,z], running, duration, reverse }) => {
      const rig = LocalRagdoll.locomotion(config);
      const frame = () => ({ bodies: Array.from(rig.body_states()) as number[], movement: Array.from(rig.movement_state()) as number[], moment: Array.from(rig.stopping_moment()) as number[] });
      const frames = [frame()];
      for (let step = 0; step < 60 + duration + reverse + 360; step++) {
        if (step === 60) rig.set_movement(x,z,running);
        if (step === 60 + duration) rig.set_movement(reverse ? -x : 0,reverse ? -z : 0,reverse > 0 && running);
        if (step === 60 + duration + reverse) rig.set_movement(0,0,false);
        rig.step(); frames.push(frame());
      }
      rig.free();
      return frames;
    });
  }, native.map(({frames: _, ...scenario}) => scenario));
  let deviation = 0, peakMoment = 0;
  expect(traces).toHaveLength(8);
  for (const [index, frames] of traces.entries()) {
    const reference = native[index]!;
    expect(frames).toHaveLength(reference.frames.length);
    for (const [tick, frame] of frames.entries()) {
      for (const [key,size] of [['bodies',210],['movement',11],['moment',3]] as const) {
        expect(frame[key]).toHaveLength(size);
        for (let i=0;i<size;i++) deviation = Math.max(deviation,Math.abs(frame[key][i]! - reference.frames[tick]![key][i]!));
      }
      expect(frame.bodies[1]).toBeGreaterThan(1.1);
      const moment = Math.hypot(...frame.moment);
      expect(moment).toBeLessThanOrEqual(120 * frame.movement[10]! + 0.001);
      peakMoment = Math.max(peakMoment,moment);
    }
    const last = frames.at(-1)!;
    expect(Math.hypot(...last.bodies.slice(35,38))).toBeLessThan(0.2);
    expect(last.movement[10]).toBe(0);
    expect(last.moment).toEqual([0,0,0]);
  }
  expect(peakMoment).toBeGreaterThan(100);
  expect(deviation).toBeLessThan(1e-5);
});

test('visible stopping effort rises during braking and clears after settling and reset', async ({page}) => {
  await page.goto('/playground/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  await page.getByLabel('Test scene').selectOption('locomotion');
  await expect(page.locator('#stopping-moment')).toHaveText('0.00 N·m');
  await expect(page.locator('#support-moment')).toHaveText('0.00 N·m');
  await page.getByRole('button',{name:'Resume',exact:true}).click();
  await expect.poll(async()=>Number(await page.locator('#tick').textContent())).toBeGreaterThan(60);
  await page.getByRole('button',{name:'Focus movement controls'}).click();
  await page.keyboard.down('d');
  await expect.poll(async()=>Number(await page.locator('#position-x').textContent())).toBeGreaterThan(1);
  await page.keyboard.up('d');
  await expect(page.locator('#movement-command')).toHaveText('Idle');
  await expect.poll(async()=>parseFloat((await page.locator('#stopping-moment').textContent())!)).toBeGreaterThan(1);
  await expect(page.locator('#movement-strength')).toHaveText('0%',{timeout:15_000});
  await expect(page.locator('#stopping-moment')).toHaveText('0.00 N·m');
  await expect(page.locator('#support-moment')).toHaveText('0.00 N·m');
  await page.getByRole('button',{name:'Pause',exact:true}).click();
  await page.getByRole('button',{name:'Reset fixture',exact:true}).click();
  await expect(page.locator('#stopping-moment')).toHaveText('0.00 N·m');
  await expect(page.locator('#support-moment')).toHaveText('0.00 N·m');
});
