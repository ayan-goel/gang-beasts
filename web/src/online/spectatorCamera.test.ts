import { expect, it } from 'vitest';
import { followedFighter, nextSurvivor } from './spectatorCamera';

const match = { epoch:'48', phase:'playing' as const, visible:[true, false, true, true] };

it('cycles through survivors in either direction and wraps past eliminated slots', () => {
  expect(nextSurvivor(match.visible, null, 1)).toBe(0);
  expect(nextSurvivor(match.visible, null, -1)).toBe(3);
  expect(nextSurvivor(match.visible, 0, 1)).toBe(2);
  expect(nextSurvivor(match.visible, 2, -1)).toBe(0);
  expect(nextSurvivor(match.visible, 3, 1)).toBe(0);
  expect(nextSurvivor([false, false], 0, 1)).toBeNull();
});

it('keeps a spectator choice within its round and follows the next survivor after a fall', () => {
  const choice = {epoch:'48', slot:2};
  expect(followedFighter(match, null, choice)).toBe(2);
  expect(followedFighter(match, 1, choice)).toBe(2);
  expect(followedFighter(match, 0, choice)).toBeNull();
  expect(followedFighter(match, null, null)).toBeNull();
  expect(followedFighter({...match, visible:[true,false,false,true]}, null, choice)).toBe(3);
  expect(followedFighter({...match, visible:[false,false,false,false]}, null, choice)).toBeNull();
  expect(followedFighter({...match, epoch:'49'}, null, choice)).toBeNull();
  expect(followedFighter({...match, phase:'match_results'}, null, choice)).toBeNull();
});
