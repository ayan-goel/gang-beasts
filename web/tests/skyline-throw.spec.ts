import { expect, test } from '@playwright/test';
import { measureFighter } from './support/liftTrace';
import type { LocalDuel } from '../src/generated/wasm/brawler_wasm';

test('a running grab and lift throws Blue off the east roof while Coral stays in', async ({page}) => {
  await page.goto('/play/');
  await expect(page.getByRole('button', {name: 'Start match'})).toBeEnabled();
  const results = await page.evaluate(async () => {
    const wasm = await import(/* @vite-ignore */ String('/src/generated/wasm/brawler_wasm.js'));
    const config = await (await fetch('/src/generated/physics.json')).text();
    return [true, false].map(lift => {
      const game: LocalDuel = wasm.LocalDuel.skyline_match(config);
      const step = (ticks: number) => { for (let i = 0; i < ticks; i++) game.step(); };
      const pelvis = (slot: number) => game.body_states(slot).slice(28,31);
      const gripping = () => [0,1].some(hand => game.grip_region(0, hand) !== undefined);
      try {
        step(180);
        game.set_input(1, 1, 0, false);
        for (let i = 0; i < 700 && pelvis(1)[0]! < 8.3; i++) game.step();
        game.set_input(1, 0, 0, false);
        step(60);
        game.set_hand(0, 0, true);
        game.set_hand(0, 1, true);
        for (let i = 0; i < 620 && !gripping(); i++) {
          if (i % 3 === 0) {
            const a = pelvis(0), b = pelvis(1);
            const dx = b[0]!-a[0]!, dz = b[2]!-a[2]!, length = Math.hypot(dx,dz);
            game.set_input(0, Math.round(127*dx/length)/127, Math.round(127*dz/length)/127, true);
          }
          game.step();
        }
        game.set_lift(0, lift);
        game.set_input(0, 1, 0, false);
        step(34);
        const held = gripping(), release = [...game.body_states(1)];
        game.set_hand(0, 0, false);
        game.set_hand(0, 1, false);
        game.set_lift(0, false);
        const stableRelease = [...game.body_states(1)].every((v,i) => v === release[i]);
        game.set_input(0, -1, 0, true);
        step(35);
        game.set_input(0, 0, 0, false);
        for (let i = 35; i < 260 && JSON.parse(game.match_json()!).phase === 'playing'; i++) game.step();
        return {held, release, stableRelease, match: JSON.parse(game.match_json()!)};
      } finally { game.free(); }
    });
  });
  const [thrown, baseline] = results;
  for (const result of results) {
    expect(result.held).toBe(true);
    expect(result.stableRelease).toBe(true);
  }
  const clearance = (states: number[]) => measureFighter(index => {
    const i = index*14;
    return {position: [states[i]!, states[i+1]!, states[i+2]!],
      rotation: [states[i+3]!, states[i+4]!, states[i+5]!, states[i+6]!]};
  }).clearance;
  expect(clearance(thrown!.release)).toBeGreaterThan(.10);
  expect(clearance(baseline!.release)).toBeLessThan(.01);
  expect(thrown!.release[28]).toBeLessThan(10);
  expect(thrown!.release[35]).toBeGreaterThan(2);
  expect(thrown!.match).toMatchObject({phase: 'round_results', scores: [1,0], visible: [true,false]});
  expect(baseline!.match).toMatchObject({phase: 'playing', scores: [0,0], visible: [true,true]});
});
