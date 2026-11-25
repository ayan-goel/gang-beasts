import { expect, test } from '@playwright/test';
import { execFileSync } from 'node:child_process';

type Frame = { bodies: number[]; movement: number[] };
type Trace = { running: boolean; gap: number; frames: Frame[] };

test('WASM preserves native reversal traces across keyboard neutral gaps', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  const traces: Trace[] = await page.evaluate(async () => {
    const { default: init, LocalRagdoll } = await import(/* @vite-ignore */ String('/src/generated/wasm/brawler_wasm.js'));
    await init();
    const configuration = await (await fetch('/src/generated/physics.json')).text();
    const traces = [];
    for (const running of [false, true]) {
      for (const gap of [0, 1, 5]) {
        const rig = LocalRagdoll.locomotion(configuration);
        const frame = () => ({ bodies: Array.from(rig.body_states()) as number[], movement: Array.from(rig.movement_state()) as number[] });
        const frames = [frame()];
        for (let step = 0; step < 420; step++) {
          if (step >= 60) {
            const leg = Math.floor((step - 60) / 45), tick = (step - 60) % 45;
            const sign = leg % 2 === 0 ? 1 : -1;
            if (leg > 0 && tick === 0) rig.set_movement(0, 0, false);
            const request = leg > 0 && tick < gap ? 0 : sign;
            rig.set_movement(request, 0, running && request !== 0);
          }
          rig.step();
          frames.push(frame());
        }
        rig.free();
        traces.push({ running, gap, frames });
      }
    }
    return traces;
  });
  const native: Trace[] = JSON.parse(execFileSync('cargo', ['run', '--quiet', '--release', '--locked', '-p', 'brawler-engine', '--example', 'reversal_trace'], {
    cwd: '..', encoding: 'utf8', maxBuffer: 32 * 1024 * 1024,
  }));
  expect(traces).toHaveLength(6);
  let deviation = 0;
  for (const [index, trace] of traces.entries()) {
    const expected = native[index]!;
    expect([trace.running, trace.gap]).toEqual([expected.running, expected.gap]);
    expect(trace.frames).toHaveLength(421);
    for (const [tick, frame] of trace.frames.entries()) {
      for (const [key, size] of [['bodies', 210], ['movement', 11]] as const) {
        expect(frame[key]).toHaveLength(size);
        for (let item = 0; item < size; item++) {
          deviation = Math.max(deviation, Math.abs(frame[key][item]! - expected.frames[tick]![key][item]!));
        }
      }
      expect(frame.bodies[1]).toBeGreaterThan(1.1);
      if (tick > 60 && (tick - 60) % 45 === 0) {
        const leg = (tick - 60) / 45 - 1;
        expect(frame.bodies[2 * 14 + 7]! * (leg % 2 === 0 ? 1 : -1)).toBeGreaterThan(0.5);
      }
    }
  }
  expect(deviation).toBeLessThan(1e-5);
});
