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

const compact = readFileSync(new URL('../../../tests/fixtures/protocol/compact-skyline-v1.bin', import.meta.url));
it('decodes compact native frames with exact positions and sub-0.01 degree rotation error', () => {
  const before = decodeSnapshot(golden);
  const after = decodeSnapshot(compact);
  expect({...after, bodies: undefined}).toEqual({...before, bodies: undefined});
  const rotations = [[0,0,0,-1],[-0.5,0.5,-0.5,0.5],[0,-0.8,0,0.6],[0.6,0,0.8,0]];
  let i = 0;
  for (const [id, pose] of after.bodies) {
    expect(pose.position).toEqual(before.bodies.get(id)?.position);
    const original = rotations[i++ % 4]!;
    const dot = pose.rotation.reduce((sum, v, j) => sum + v * original[j]!, 0);
    expect(2 * Math.acos(Math.min(1, Math.abs(dot))) * 180 / Math.PI).toBeLessThan(0.01);
  }
  expect(i).toBe(31);
});

it('rejects invalid compact indices, impossible rotations, reserved integers and malformed lengths', () => {
  for (const mutate of [
    (b: Buffer) => { b[36] = 4; },
    (b: Buffer) => { b.writeInt16LE(32767, 37); },
    (b: Buffer) => { b.writeInt16LE(-32768, 39); },
    (b: Buffer) => { b.fill(0x7f, 37, 43); },
    (b: Buffer) => { b[2] = 3; },
  ]) {
    const copy = Buffer.from(compact);
    mutate(copy);
    expect(() => decodeSnapshot(copy)).toThrow();
  }
  expect(() => decodeSnapshot(compact.subarray(0,-1))).toThrow();
});

it('decodes opted-in landing records and rejects future or partial events', () => {
  const bytes = Buffer.from(compact);
  const block = 20 + 31 * 23;
  bytes[block] = 2;
  const first = block + 24;
  bytes[first+5] = 50;
  bytes.writeUInt16LE(7196,first+6);
  const state = decodeSnapshot(bytes);
  expect(state.fighters[0]?.landing).toEqual({tick:7196,impulse:50});
  expect(state.fighters[1]?.landing).toBeUndefined();
  bytes.writeUInt16LE(7202,first+6);
  expect(() => decodeSnapshot(bytes)).toThrow();
  bytes.writeUInt16LE(0,first+6);
  expect(() => decodeSnapshot(bytes)).toThrow();
  bytes.writeUInt16LE(7200,first+6);
  bytes[first+5] = 0;
  expect(() => decodeSnapshot(bytes)).toThrow();
});

it('reconstructs the precise step tick from the recent contact and slot in compact block three', () => {
  const bytes = Buffer.from(compact);
  const block = 20 + 31 * 23;
  bytes[block] = 3;
  bytes[block+48] = 13 * 8 + 1;
  expect(decodeSnapshot(bytes).fighters[1]?.footstep).toBe(7189);
  expect(decodeSnapshot(bytes).fighters[0]?.footstep).toBeUndefined();
  bytes[block+48] = 14 * 8 + 1;
  expect(() => decodeSnapshot(bytes)).toThrow();
  bytes[block+48] = 8;
  expect(() => decodeSnapshot(bytes)).toThrow();
  bytes[block+48] = 3 * 8 + 1;
  bytes.writeUInt32LE(2,block+20);
  expect(() => decodeSnapshot(bytes)).toThrow();
});
