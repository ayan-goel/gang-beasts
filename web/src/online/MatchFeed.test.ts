import { readFileSync } from 'node:fs';
import { describe, expect, it } from 'vitest';
import { MatchFeed } from './MatchFeed';

function roster(slots: number[], own: number) {
  return Uint8Array.of(26, 1, own, slots.length, ...slots.flatMap(slot => [slot, 7-slot, 1, 65+slot]));
}

describe('online match assignment', () => {
  it('binds a compact native index even when lobby seats have gaps', () => {
    const game = new MatchFeed(roster([1, 4, 7], 2), () => {});
    expect(game.fighters.map(p => p.slot)).toEqual([1, 4, 7]);
    expect(game.fighters[game.own!]).toEqual({slot:7, color:0, name:'H'});
    const spectator = new MatchFeed(roster([0,1,2,3,4,5,6,7], 255), () => {});
    expect(spectator.own).toBeNull();
    expect(spectator.fighters).toHaveLength(8);
  });
  it('rejects a snapshot for another roster before changing epoch or acknowledging it', () => {
    const frames: Uint8Array[] = [];
    const game = new MatchFeed(roster([1, 4, 7], 2), bytes => frames.push(bytes));
    const duel = readFileSync(new URL('../../../tests/fixtures/protocol/full-skyline-v1.bin', import.meta.url));
    expect(() => game.receive(duel)).toThrow('Snapshot roster changed');
    expect(game.latest).toBeNull();
    expect(frames).toEqual([]);
  });
  it('rejects missing, duplicate, out-of-order and unowned identities', () => {
    const valid = roster([1, 4, 7], 2);
    for (const packet of [valid.slice(0, -1), Uint8Array.of(...valid, 0),
      roster([1,1,4], 0), roster([4,1,7], 0), roster([1,4,7], 3), roster([0], 0)]) {
      expect(() => new MatchFeed(packet, () => {})).toThrow();
    }
  });
});
