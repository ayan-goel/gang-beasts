import { isWinTarget } from './api';

interface Player { slot: number; name: string; color: number; host: boolean }
export interface LiveRoom {
  revision: number; slot: number; occupied: number; ready: number; host: number;
  wins: number; capacity: number; players: Player[];
}

export function decodeLobbyState(bytes: Uint8Array, previous: LiveRoom | null): LiveRoom {
  const welcome = !previous;
  if (bytes.length < (welcome ? 10 : 9) || bytes[0] !== (welcome ? 17 : 19) || bytes[1] !== 1) {
    throw new RangeError('Invalid lobby packet');
  }
  const slot = previous?.slot ?? bytes[2]!;
  const offset = welcome ? 3 : 2;
  const occupied = bytes[offset]!, ready = bytes[offset + 1]!, host = bytes[offset + 2]!;
  if (slot > 7 || host > 7 || !(occupied & (1 << slot)) || !(occupied & (1 << host)) || (ready & ~occupied)) {
    throw new RangeError('Invalid lobby seats');
  }
  const revision = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength).getUint32(offset + 3, true);
  let cursor = offset + 7;
  if (previous?.revision === revision) {
    if (bytes.length !== cursor || occupied !== previous.occupied || host !== previous.host) throw new RangeError('Invalid roster revision');
    return {...previous, ready};
  }
  const capacity = bytes[cursor++] ?? 0;
  if (capacity < 2 || capacity > 8 || occupied >= (1 << capacity)) throw new RangeError('Invalid lobby capacity');
  const players: Player[] = [];
  const decoder = new TextDecoder('utf-8', {fatal: true});
  for (let i = 0; i < capacity; i++) {
    if (!(occupied & (1 << i))) continue;
    const color = bytes[cursor++], size = bytes[cursor++];
    if (color === undefined || color > 7 || !size || size > 96 || cursor + size > bytes.length) throw new RangeError('Invalid lobby player');
    const name = decoder.decode(bytes.subarray(cursor, cursor + size));
    players.push({slot: i, name, color, host: host === i});
    cursor += size;
  }
  const wins = cursor === bytes.length ? 3 : bytes[cursor++];
  if (!isWinTarget(wins) || cursor !== bytes.length) throw new RangeError('Invalid roster length');
  return {slot, occupied, ready, host, revision, wins, capacity, players};
}
