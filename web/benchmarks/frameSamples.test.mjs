import assert from 'node:assert/strict';
import test from 'node:test';
import { splitCapture, summarize, unpack } from './frameSamples.mjs';

test('nearest-rank frame statistics retain a missed refresh and unrounded tail', () => {
  const samples = [...Array(94).fill(16.6), ...Array(5).fill(16.7), 54.3];
  assert.deepEqual(summarize(samples), {count:100, p50:16.6, p95:16.7, p99:16.7, max:54.3});
  assert.throws(() => summarize([]));
});

test('trace framing requires consecutive 30 Hz snapshots and complete payloads', () => {
  const record = tick => {
    const value = Buffer.alloc(12);
    value.writeUInt32LE(8); value[4] = 2; value.writeUInt32LE(tick, 8);
    return value;
  };
  const trace = Buffer.concat([record(2), record(4)]);
  assert.deepEqual(unpack(trace).map(p => p.readUInt32LE(4)), [2,4]);
  assert.throws(() => unpack(trace.subarray(0,-1)));
  assert.throws(() => unpack(Buffer.concat([record(2), record(6)])));
  assert.throws(() => unpack(Buffer.alloc(0)));
});

test('capture metadata identifies the native engine and map used to make the packets', () => {
  const identity = 'a'.repeat(64) + 'b'.repeat(64);
  const payload = Buffer.from([19,0,0,0]);
  const result = splitCapture(Buffer.concat([Buffer.from(identity),payload]));
  assert.equal(result.identity, identity);
  assert.deepEqual(result.bytes,payload);
  assert.throws(() => splitCapture(Buffer.alloc(130,0)));
});
