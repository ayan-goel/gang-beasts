import { readFileSync } from 'node:fs';
import { expect, it } from 'vitest';
import { decodeSnapshot } from './snapshot';

const golden = readFileSync(new URL('../../../tests/fixtures/protocol/full-skyline-v1.bin', import.meta.url));

it('decodes the native full snapshot including an exact wide epoch and valid zero ack', () => {
  const snap = decodeSnapshot(golden);
  expect(snap.tick).toBe(0xfffffff0);
  expect(snap.applied).toBe(0);
  expect(snap.round).toBe(0x20000000000001n);
  expect(snap.arenaTick).toBe(7201);
  expect(snap.winner).toBeNull();
  expect(snap.fighters[0]).toEqual({ consciousness: 0, score: 1, hands: [5, 4], health: 76.5, stamina: 41.25, grips: [65543, 0] });
  expect(snap.bodies.get(65543)?.position).toEqual([65543 / 8, -1.25, 3]);
});

it('rejects invalid extents, flags, rotations, IDs, gameplay state and grip references', () => {
  const game = 20 + 31 * 32;
  for (const [offset, value] of [[0, 9], [1, 3], [2, 4], [12, 1], [16, 255], [18, 0], [20, 2], [48, 0],
  [game, 2], [game + 1, 8], [game + 2, 8], [game + 3, 9], [game + 6, 1], [game + 24, 1], [game + 25, 5],
  [game + 29, 1], [game + 40, 1]] as const) {
    const copy = Buffer.from(golden);
    if (offset === 48) copy.writeFloatLE(value, offset);
    else copy[offset] = value;
    expect(() => decodeSnapshot(copy), 'offset ' + offset).toThrow();
  }
  for (const bytes of [golden.subarray(1), golden.subarray(0, -1), Buffer.concat([golden, Buffer.alloc(1)])]) {
    expect(() => decodeSnapshot(bytes)).toThrow();
  }
});
