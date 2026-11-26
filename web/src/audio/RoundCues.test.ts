import { expect, it } from 'vitest';
import { RoundCues } from './RoundCues';
import type { MatchView } from '../rendering/SkylineView';

function observeRound() {
  const cursor = new RoundCues();
  let time = 0;
  return {cursor, at(phase: MatchView['phase'], remaining: number, epoch = '1', winner: number | null = null, elapsed = 100) {
    time += elapsed;
    return cursor.observe({phase, remaining, epoch, winner}, time);
  }};
}

it('follows displayed count changes, play, score, the next round and match results once', () => {
  const {at} = observeRound();
  expect(at('countdown',178)).toBeUndefined();
  expect(at('countdown',121)).toBeUndefined();
  expect(at('countdown',120)).toBe('count');
  expect(at('countdown',119)).toBeUndefined();
  expect(at('countdown',60)).toBe('count');
  expect(at('playing',0)).toBe('go');
  expect(at('playing',0)).toBeUndefined();
  expect(at('round_results',300,'1',0)).toBe('round_win');
  expect(at('round_results',1,'1',0)).toBeUndefined();
  expect(at('countdown',179,'2')).toBe('count');
  expect(at('playing',0,'2')).toBe('go');
  expect(at('round_results',300,'2',1)).toBe('round_win');
  expect(at('match_results',0,'2',1)).toBe('match_win');
  expect(at('match_results',0,'2',1)).toBeUndefined();
});

it('uses a separate draw cue and takes a silent baseline after reload, pause or a delivery gap', () => {
  const {cursor,at} = observeRound();
  expect(at('playing',0)).toBeUndefined();
  expect(at('round_results',300)).toBe('draw');
  cursor.reset();
  expect(at('round_results',200)).toBeUndefined();
  expect(at('countdown',65,'2',null,1200)).toBeUndefined();
  expect(at('countdown',60,'2')).toBe('count');
  expect(at('playing',0,'2',null,800)).toBeUndefined();
  cursor.reset();
  expect(at('match_results',0,'2',1)).toBeUndefined();
});
