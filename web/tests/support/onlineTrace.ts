import type { Page } from '@playwright/test';
import { decodeSnapshot } from '../../src/net/snapshot';

export function observeMatch(page: Page) {
  let axes = [0, 0];
  const history: object[] = [];
  page.on('websocket', socket => {
    if (!new URL(socket.url()).pathname.endsWith('/ws')) return;
    socket.on('framesent', ({payload}) => {
      if (Buffer.isBuffer(payload) && payload[0] === 1 && payload.length === 18) {
        axes = [payload.readInt8(10), payload.readInt8(11)];
      }
    });
    let last = -60;
    socket.on('framereceived', ({payload}) => {
      if (!Buffer.isBuffer(payload) || payload[0] !== 2) return;
      const tick = payload.readUInt32LE(4);
      if (tick - last < 60) return;
      last = tick;
      const frame = decodeSnapshot(payload);
      const positions = [1,17].map(id => frame.bodies.get(id)!.position.map(v=>Number(v.toFixed(2))));
      history.push({tick,round: frame.number, phase: frame.phase, applied: frame.applied, axes, positions});
      if (history.length > 24) history.shift();
    });
  });
  return history;
}
