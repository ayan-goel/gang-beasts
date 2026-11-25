import { describe, expect, it } from 'vitest';
import golden from '../../../tests/fixtures/protocol/input-v1.json';
import { encodeInput, quantizeDirection } from './inputFrame';

describe('v1 input records', () => {
  it('matches the shared native golden bytes including signed axes and short taps', () => {
    expect([...encodeInput({ ...golden, axes: [-127, 127] })]).toEqual(golden.bytes);
  });
  it('rejects values that DataView would otherwise truncate or wrap', () => {
    const valid = { ...golden, axes: [0, 0] as const };
    for (const fields of [{ sequence: -1 }, { tick: 2 ** 32 }, { tick: 1.2 }, { axes: [-128, 0] as const },
      { held: 1024 }, { released: 65536 }, { pressed: 2 ** 32 }, { axes: [NaN, 0] as const },
      { held: 0, released: 0, pressed: 1 }, { held: 1, released: 1, pressed: 0 }]) {
      expect(() => encodeInput({ ...valid, ...fields })).toThrow();
    }
  });
  it('retains analog magnitude and caps diagonal and large movement', () => {
    expect(quantizeDirection(0, 0)).toEqual([0, 0]);
    expect(quantizeDirection(0.5, -0.5)).toEqual([64, -64]);
    expect(quantizeDirection(1, 1)).toEqual([90, 90]);
    expect(quantizeDirection(-1e300, 0)).toEqual([-127, 0]);
    expect(() => quantizeDirection(Infinity, 0)).toThrow();
  });
});
