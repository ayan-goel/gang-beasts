import { describe, expect, it } from 'vitest';
import { decodeLobbyState } from './liveRoom';

describe('lobby control protocol v1', () => {
  const welcome = Uint8Array.of(17, 1, 7, 129, 128, 0, 120, 86, 52, 18, 8, 0, 1, 65, 7, 1, 66);
  const update = Uint8Array.of(19, 1, 129, 1, 0, 120, 86, 52, 18);

  it('decodes the recipient, ready seats, host and little-endian roster revision', () => {
    const current = decodeLobbyState(welcome, null);
    expect(current).toMatchObject({slot: 7, occupied: 129, ready: 128, host: 0, revision: 0x12345678});
    expect(current.players).toEqual([{slot: 0, color: 0, name: 'A', host: true}, {slot: 7, color: 7, name: 'B', host: false}]);
    expect(decodeLobbyState(update, current)).toEqual({...current, ready: 1});
  });

  it('rejects an update before welcome, a second welcome and invalid ownership or lengths', () => {
    const current = decodeLobbyState(welcome, null);
    expect(() => decodeLobbyState(update, null)).toThrow();
    expect(() => decodeLobbyState(welcome, current)).toThrow();
    for (const [offset, value] of [[1, 2], [2, 8], [3, 1], [4, 2], [5, 1]] as const) {
      const bad = welcome.slice();
      bad[offset] = value;
      expect(() => decodeLobbyState(bad, null)).toThrow();
    }
    expect(() => decodeLobbyState(update.subarray(0, 8), current)).toThrow();
    expect(() => decodeLobbyState(welcome.subarray(0, 14), null)).toThrow();
    expect(() => decodeLobbyState(Uint8Array.from([...welcome, 1]), null)).toThrow();
  });
});
