import { decodeSnapshot, type Snapshot } from '../net/snapshot';

import { SnapshotTimeline } from './SnapshotTimeline';

export interface Fighter { slot: number; color: number; name: string }

export class MatchFeed {
  latest: Snapshot | null = null;
  readonly timeline = new SnapshotTimeline();
  onRoundChange?: () => void;
  onSnapshot?: () => void;
  readonly fighters: readonly Fighter[];
  readonly own: number | null;

  constructor(packet: Uint8Array, readonly transmit: (packet: Uint8Array) => void) {
    const duel = packet[0] === 23;
    const count = duel ? 2 : packet[3];
    const own = packet[2];
    if ((!duel && packet[0] !== 26) || packet[1] !== 1 || count === undefined ||
        count < (duel ? 2 : 3) || count > 8 || own === undefined || (own !== 255 && own >= count)) {
      throw new Error('Invalid match assignment');
    }
    this.own = own === 255 ? null : own;
    let offset = duel ? 3 : 4;
    const read = (): Fighter => {
      const slot = packet[offset++], color = packet[offset++], size = packet[offset++];
      if (slot === undefined || slot > 7 || color === undefined || color > 7 || !size || size > 96 || offset + size > packet.length) throw new Error('Invalid fighter identity');
      const name = new TextDecoder('utf-8', {fatal: true}).decode(packet.subarray(offset, offset + size));
      offset += size;
      return {slot, color, name};
    };
    this.fighters = Array.from({length:count}, read);
    if (offset !== packet.length || this.fighters.some((p,i) => i > 0 && this.fighters[i-1]!.slot >= p.slot)) {
      throw new Error('Invalid starting roster');
    }
  }

  receive(packet: Uint8Array) {
    const frame = decodeSnapshot(packet);
    if (frame.fighters.length !== this.fighters.length) throw new Error('Snapshot roster changed');
    if (!this.timeline.push(frame, performance.now())) return false;
    if (this.latest?.round !== frame.round) {
      this.onRoundChange?.();
      const ack = new Uint8Array(10);
      ack.set([24, 1]);
      new DataView(ack.buffer).setBigUint64(2, frame.round, true);
      this.transmit(ack);
    }
    this.latest = frame;
    this.onSnapshot?.();
    return true;
  }
}
