import { readFileSync } from 'node:fs';
import { expect, it, vi } from 'vitest';
import { bindRemoteControls } from './RemoteControls';
import { GroupCamera } from '../rendering/GroupCamera';
import { MatchFeed } from './MatchFeed';

it('samples focused controls from network publications even when window timers never fire', () => {
  vi.useFakeTimers();
  const canvas = Object.assign(new EventTarget(), {setAttribute() {}, focus() {}});
  const doc = Object.assign(new EventTarget(), {activeElement: canvas, hidden: false});
  vi.stubGlobal('document', doc);
  vi.stubGlobal('window', Object.assign(new EventTarget(), {}));
  const frames: Uint8Array[] = [];
  const feed = new MatchFeed(Uint8Array.of(23,1,0,0,0,1,65,1,1,1,66), frame => frames.push(frame));
  const world = readFileSync(new URL('../../../tests/fixtures/protocol/full-skyline-v1.bin', import.meta.url));
  const gameplay = 20 + 32 * world.readUInt16LE(16);
  const publish = (epoch: bigint, phase: number) => {
    world.writeUInt32LE((world.readUInt32LE(4) + 2) >>> 0, 4);
    world[gameplay + 1] = phase;
    world.writeBigUInt64LE(epoch, gameplay + 8);
    feed.receive(world);
  };
  const down = (code: string) => canvas.dispatchEvent(Object.assign(new Event('keydown'), {code}));
  const stop = bindRemoteControls(canvas as unknown as HTMLCanvasElement, feed, new GroupCamera());
  try {
    publish(1n, 1);
    down('KeyW');
    vi.advanceTimersByTime(34);
    publish(1n, 1);
    expect(new DataView(frames.at(-1)!.buffer).getInt8(11)).toBeLessThan(0);
    publish(2n, 0);
    publish(2n, 1);
    down('KeyD');
    vi.advanceTimersByTime(34);
    publish(2n, 1);
    const next = new DataView(frames.at(-1)!.buffer);
    expect(next.getUint32(2,true)).toBe(1);
    expect(next.getInt8(10)).toBeGreaterThan(120);
    expect(next.getInt8(11)).toBeGreaterThan(-30);
  } finally {
    stop();
    vi.unstubAllGlobals();
    vi.useRealTimers();
  }
});

it('lets a spectator turn the view without sending fighter inputs and releases its key listener', () => {
  const canvas = Object.assign(new EventTarget(), {setAttribute() {}});
  vi.stubGlobal('document', {hidden:false});
  const transmit = vi.fn();
  const feed = new MatchFeed(Uint8Array.of(23,1,255,0,0,1,65,1,1,1,66), transmit);
  const camera = new GroupCamera();
  const turn = vi.spyOn(camera,'turn');
  const stop = bindRemoteControls(canvas as unknown as HTMLCanvasElement, feed, camera);
  const key = (code: string, repeat = false) => canvas.dispatchEvent(Object.assign(new Event('keydown'), {code,repeat}));
  try {
    key('KeyX'); key('KeyX',true); key('KeyW'); key('KeyZ');
    expect(turn.mock.calls).toEqual([[1],[-1]]);
    expect(transmit).not.toHaveBeenCalled();
  } finally { stop(); vi.unstubAllGlobals(); }
  key('KeyX');
  expect(turn).toHaveBeenCalledTimes(2);
});
