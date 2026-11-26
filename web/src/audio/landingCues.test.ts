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

it('sounds the nearest new step once and gives a simultaneous landing priority', () => {
  const cursor = new MatchCues();
  const frames = (tick:number, now:number, landing = false) => cursor.observe('1','quiet',['active','active'],now,
    [8,4].map((distance,slot) => ({distance,pan:slot,health:100,footstep:tick,landing:landing ? {tick,impulse:32} : undefined})));
  expect(frames(8,0)).toEqual({});
  expect(frames(20,100)).toEqual({footstep:{tick:20,pan:1,distance:4}});
  expect(frames(20,200)).toEqual({});
  expect(frames(21,225,true)).toEqual({landing:{impulse:32,pan:0,distance:8}});
  expect(frames(21,250)).toEqual({});
  expect(frames(90,1400)).toEqual({});
  cursor.reset();
  expect(frames(120,1550)).toEqual({});
});
