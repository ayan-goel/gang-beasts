import { decodeSnapshot, type Snapshot } from '../net/snapshot';

export interface Fighter { slot: number; color: number; name: string }

export class MatchFeed {
  latest: Snapshot | null = null;
  readonly fighters: [Fighter, Fighter];
  readonly own: number | null;

  constructor(packet: Uint8Array) {
    if (packet[0] !== 23 || packet[1] !== 1 || ![0, 1, 255].includes(packet[2]!)) throw new Error('Invalid match assignment');
    this.own = packet[2] === 255 ? null : packet[2]!;
    let offset = 3;
    const read = (): Fighter => {
      const slot = packet[offset++], color = packet[offset++], size = packet[offset++];
      if (slot === undefined || slot > 7 || color === undefined || color > 7 || !size || size > 96 || offset + size > packet.length) throw new Error('Invalid fighter identity');
      const name = new TextDecoder('utf-8', {fatal: true}).decode(packet.subarray(offset, offset + size));
      offset += size;
      return {slot, color, name};
    };
    this.fighters = [read(), read()];
    if (this.fighters[0].slot >= this.fighters[1].slot || offset !== packet.length) throw new Error('Invalid starting roster');
  }

  receive(packet: Uint8Array) { this.latest = decodeSnapshot(packet); }
}
