import { expect, it } from 'vitest';
import { MatchCues } from './MatchCues';

it('coalesces simultaneous knockouts and keeps separate hazard and condition cues', () => {
  const sounds = new MatchCues();
  sounds.observe('1','quiet',['active','active','active'],0);
  expect(sounds.observe('1','west_warning',['knocked_out','knocked_out','active'],50))
    .toEqual({warning:'west_warning', condition:'knocked_out'});
  expect(sounds.observe('1','west_warning',['knocked_out','knocked_out','active'],100)).toEqual({});
  expect(sounds.observe('1','west_warning',['recovering','knocked_out','knocked_out'],150)).toEqual({condition:'knocked_out'});
  expect(sounds.observe('1','west_warning',['active','recovering','eliminated'],200)).toEqual({condition:'recovering'});
  expect(sounds.observe('1','west_warning',['active','active','eliminated'],250)).toEqual({});
});

it('shows current condition silently on arrival and only sounds fresh, observed transitions', () => {
  const sounds = new MatchCues();
  expect(sounds.observe('1','east_warning',['knocked_out'],0)).toEqual({});
  expect(sounds.observe('1','east_warning',['recovering'],100)).toEqual({condition:'recovering'});
  expect(sounds.observe('1','gust_warning',['knocked_out'],1600)).toEqual({});
  sounds.reset();
  expect(sounds.observe('1','gust_warning',['recovering'],1650)).toEqual({});
  expect(sounds.observe('2','quiet',['knocked_out'],1700)).toEqual({});
  expect(sounds.observe('2','quiet',['eliminated'],1800)).toEqual({});
});

it('emits each live transition once and starts fresh in a new round', () => {
  const cues = new MatchCues();
  expect(cues.observe('1','quiet',[],0).warning).toBeUndefined();
  expect(cues.observe('1','west_warning',[],100).warning).toBe('west_warning');
  expect(cues.observe('1','west_warning',[],200).warning).toBeUndefined();
  expect(cues.observe('1','east_warning',[],300).warning).toBe('east_warning');
  expect(cues.observe('1','west_warning',[],400).warning).toBeUndefined();
  expect(cues.observe('1','east_warning',[],500).warning).toBeUndefined();
  expect(cues.observe('2','quiet',[],600).warning).toBeUndefined();
  expect(cues.observe('2','west_warning',[],700).warning).toBe('west_warning');
});

it('uses the first state after joining, interruption or a missed publication interval as a baseline', () => {
  const cues = new MatchCues();
  expect(cues.observe('1','east_warning',[],0).warning).toBeUndefined();
  expect(cues.observe('1','gust_warning',[],4000).warning).toBeUndefined();
  expect(cues.observe('1','gust',[],4100).warning).toBe('gust');
  cues.reset();
  expect(cues.observe('2','west_warning',[],4200).warning).toBeUndefined();
  expect(cues.observe('2','east_warning',[],4300).warning).toBe('east_warning');
});
