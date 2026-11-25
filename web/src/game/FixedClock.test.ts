import { describe, expect, it } from 'vitest';
import { FixedClock } from './FixedClock';

describe('fixed simulation clock', () => {
  it('advances 60 steps for one second at different rendering rates', () => {
    for (const fps of [30, 60, 144]) {
      const clock = new FixedClock();
      let steps = 0;
      for (let frame = 0; frame < fps; frame++) steps += clock.advance(1 / fps);
      expect(steps).toBe(60);
    }
  });

  it('caps catch-up after suspension and discards excessive debt', () => {
    const clock = new FixedClock();
    expect(clock.advance(30)).toBe(4);
    expect(clock.advance(0)).toBe(0);
    expect(clock.droppedSeconds).toBeGreaterThan(29);
  });

  it('ignores invalid frame intervals', () => {
    const clock = new FixedClock();
    for (const value of [-1, NaN, Infinity]) expect(clock.advance(value)).toBe(0);
    expect(clock.advance(1 / 60)).toBe(1);
  });
});
