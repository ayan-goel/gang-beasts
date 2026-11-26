import type { Page } from '@playwright/test';

export function observeSnapshotTraffic(page: Page) {
  const sample = { firstTick: 0, lastTick: 0, frames: 0, bytes: 0, elapsedMs: 0, compact: false, complete: false };
  page.on('websocket', socket => {
    if (new URL(socket.url()).pathname !== '/ws') return;
    let start: number | undefined;
    socket.on('framereceived', ({payload}) => {
      if (sample.complete || !Buffer.isBuffer(payload)) return;
      const snapshot = payload[0] === 2;
      const tick = snapshot ? payload.readUInt32LE(4) : 0;
      if (start === undefined) {
        if (!snapshot) return;
        const gameplay = payload.length - payload.readUInt16LE(18);
        if (payload[gameplay + 1] !== 1 || payload[gameplay + 3] !== 8) return;
        start = performance.now();
        sample.firstTick = tick;
        sample.compact = (payload[2]! & 4) !== 0;
        return;
      }
      // The first snapshot sets the window boundary. Include every subsequent application record.
      sample.bytes += payload.byteLength;
      if (!snapshot) return;
      sample.frames++;
      sample.lastTick = tick;
      sample.elapsedMs = performance.now() - start;
      sample.complete = tick - sample.firstTick >= 360;
    });
  });
  return sample;
}
