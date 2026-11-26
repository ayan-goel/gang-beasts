import { expect, it } from 'vitest';
import { renderArenaScore, scoreSampleRate } from './arenaScore';

it('renders eight complete bars with bounded level, distinct phrases and a quiet loop seam', () => {
  const pcm = renderArenaScore();
  expect(pcm.length).toBe(scoreSampleRate * 16);
  let peak = 0, power = 0;
  for (const sample of pcm) {
    peak = Math.max(peak, Math.abs(sample));
    power += sample * sample;
  }
  expect(peak).toBeGreaterThan(0.025);
  expect(peak).toBeLessThan(0.1);
  expect(Math.sqrt(power / pcm.length)).toBeGreaterThan(0.008);
  expect(pcm[0]).toBe(0);
  expect(pcm.at(-1)).toBe(0);
  expect(pcm.slice(0, scoreSampleRate)).not.toEqual(pcm.slice(8 * scoreSampleRate, 9 * scoreSampleRate));
});
