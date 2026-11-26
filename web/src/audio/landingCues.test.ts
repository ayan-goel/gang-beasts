import {expect,it} from 'vitest';
import { MatchCues } from './MatchCues';

it('plays fresh landings once, coalesces a pile-up and treats resync as a silent baseline', () => {
  const cues = new MatchCues();
  const observe = (tick:number, now:number, phase:'active'|'knocked_out' = 'active') => cues.observe('1','quiet',
    [phase,phase],now,[35,80].map((impulse,slot) => ({health:100,pan:slot,distance:12,landing:{tick,impulse}})));
  expect(observe(10,0)).toEqual({});
  expect(observe(15,100).landing).toEqual({impulse:80,pan:1,distance:12});
  expect(observe(15,150)).toEqual({});
  expect(observe(60,1100)).toEqual({});
  expect(observe(60,1150)).toEqual({});
  expect(observe(90,1200,'knocked_out')).toEqual({condition:'knocked_out'});
  expect(observe(96,1400,'knocked_out').landing?.impulse).toBe(80);
});
