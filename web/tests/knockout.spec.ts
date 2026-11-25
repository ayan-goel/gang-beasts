import { execFileSync } from 'node:child_process';
import { expect, test } from '@playwright/test';

type Condition = [string, number, number, string];
interface TraceFrame {
  direction: [number, number] | null; taps: number[];
  states: Condition[]; changes: [number, string][]; bodies: number[][];
}

test('real strikes, unconscious fall and physical get-up agree with native across 781 ticks', async ({ page }) => {
  const trace: TraceFrame[] = JSON.parse(execFileSync('cargo', [
    'run','--quiet','--release','--locked','-p','brawler-engine','--example','knockout_trace',
  ], {cwd:'..',encoding:'utf8',maxBuffer:16*1024*1024}));
  await page.goto('/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  const frames = await page.evaluate(async commands => {
    const { default: init, LocalDuel } = await import(/* @vite-ignore */ String('/src/generated/wasm/brawler_wasm.js'));
    await init();
    const duel = new LocalDuel(await (await fetch('/src/generated/physics.json')).text());
    try {
      return commands.map(({ direction, taps }) => {
        if (direction) {
          duel.set_input(1, ...direction, false);
          duel.set_input(0, 0, 0, false);
        }
        for (const hand of taps) {
          duel.set_hand(0, hand, true);
          duel.set_hand(0, hand, false);
        }
        duel.step();
        const states = [0,1].map(id => {
          const v = JSON.parse(duel.condition_json(id));
          return [v.phase, v.health, v.knockouts, v.sinceTick] as Condition;
        });
        const changes = JSON.parse(duel.condition_events_json() ?? '[]') as [number,string][];
        return { states, changes, bodies: [0,1].map(id => Array.from<number>(duel.body_states(id))) };
      });
    } finally { duel.free(); }
  }, trace.map(({direction,taps}) => ({direction,taps})));
  const timeline = frames.flatMap((f, i) => f.changes.map(([id,phase]) => [i+1,id,phase]));
  expect(timeline).toEqual([[396,1,'knocked_out'],[576,1,'recovering'],[606,1,'active']]);
  let maximumBodyError = 0;
  let maximumHealthError = 0;
  expect(frames).toHaveLength(781);
  for (const [i, actual] of frames.entries()) {
    const native = trace[i]!;
    expect(actual.changes).toEqual(native.changes);
    for (const [id, state] of actual.states.entries()) {
      const [phase, hp, ko, since] = state;
      const expected = native.states[id]!;
      expect([phase,ko,since]).toEqual([expected[0],expected[2],expected[3]]);
      maximumHealthError = Math.max(maximumHealthError, Math.abs(hp-expected[1]));
      const body = actual.bodies[id]!;
      expect(body).toHaveLength(210);
      for (let j=0;j<body.length;j++) maximumBodyError = Math.max(maximumBodyError, Math.abs(body[j]!-native.bodies[id]![j]!));
    }
  }
  expect(maximumHealthError).toBeLessThan(1e-5);
  expect(maximumBodyError).toBeLessThan(1e-5);
  expect(frames[575]!.bodies[1]![29]!).toBeLessThan(0.4);
  expect(frames[780]!.bodies[1]![29]!).toBeGreaterThan(0.75);
});

test('sparring shows damage on the defender and reset restores both fighters', async ({page}) => {
  await page.goto('/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  await page.getByLabel('Test scene').selectOption('duel');
  const blue = page.getByLabel('Blue condition', {exact:true});
  await expect(blue).toHaveText('Active · 100 health');
  await page.getByRole('button', {name:'Resume',exact:true}).click();
  await expect.poll(async () => Number(await page.locator('#duel-tick').innerText())).toBeGreaterThan(59);
  await page.keyboard.press('j');
  await expect(blue).toHaveText(/Active · [0-9]{1,2} health/);
  await expect(page.getByLabel('Coral condition', {exact:true})).toHaveText('Active · 100 health');
  await page.getByRole('button',{name:'Reset encounter'}).click();
  await expect(blue).toHaveText('Active · 100 health');
});
