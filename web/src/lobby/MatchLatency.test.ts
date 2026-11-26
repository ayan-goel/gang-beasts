import { expect, it } from 'vitest';
import { MatchLatency } from './MatchLatency';

it('paces probes, ignores unrelated replies and replaces an expired probe', () => {
  const link = new MatchLatency();
  const first = link.probe(123)!;
  expect([...first]).toEqual([36,1,1,0,0,0]);
  expect(link.probe(1123)).toBeNull();
  const other = Uint8Array.of(37,1,2,0,0,0);
  expect(link.reply(other, 1150)).toBeNull();
  const next = link.probe(3123)!;
  expect(next[2]).toBe(2);
  expect(link.reply(Uint8Array.of(37,1,1,0,0,0), 3130)).toBeNull();
  expect(link.reply(other, 3183)).toBe(false);
  expect(link.reply(other, 3190)).toBeNull();
  expect(link.probe(4122)).toBeNull();
});

it('warns on sustained high RTT and clears after sustained recovery', () => {
  const link = new MatchLatency();
  const samples = [65, 220, 62, 205, 210, 152, 145, 82, 80];
  const flags = samples.map((ms, i) => {
    const reply = link.probe(i * 1000)!;
    reply[0] = 37;
    return link.reply(reply, i * 1000 + ms);
  });
  expect(flags).toEqual([false,false,false,true,true,true,true,true,false]);
});
