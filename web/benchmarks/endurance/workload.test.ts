import { expect, test } from 'vitest';
import { chooseInput, verifyGrips } from './workload';
import type { FighterState } from '../../src/net/snapshot';

const fighter = (): FighterState => ({consciousness:0,score:0,hands:[0,0],grips:[0,0],health:100,stamina:100});

test('a scripted match uses both hands, converges on the center and leaves one survivor', () => {
  const early = Array.from({length:24},(_,pulse) => chooseInput(0,1,6*pulse,0,-4,3));
  expect(early.every(f => f.axes[0] > 0 && f.axes[1] < 0)).toBe(true);
  expect(early.some(f => f.held & 4)).toBe(true);
  expect(early.some(f => f.held & 8)).toBe(true);
  const exits = Array.from({length:8},(_,slot) => chooseInput(0,1,500,slot,2,3));
  expect(exits.filter(f => f.held & 2)).toHaveLength(7);
  expect(exits[0]!.axes).toEqual([0,0]);
  expect(chooseInput(1,1,10800,4,4,4).axes[0]).toBeLessThan(0);
});

test('a published grip must connect two live fighters or a live owner and a prop', () => {
  const fighters = Array.from({length:8},fighter);
  fighters[0]!.grips = [17,65540];
  expect(verifyGrips(fighters)).toBe(2);
  fighters[1]!.consciousness = 3;
  expect(() => verifyGrips(fighters)).toThrow(/eliminated/);
  fighters[0]!.grips = [0,65540];
  fighters[0]!.consciousness = 3;
  expect(() => verifyGrips(fighters)).toThrow(/eliminated/);
});
