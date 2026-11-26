import { expect, it } from 'vitest';
import mapJson from '../../../assets/maps/skyline/map.json?raw';
import type { SkylineContent } from '../lobby/content';
import { MatchCues } from './MatchCues';
import { recentBreakawayMount } from './breakawaySound';

it('uses the roof-side attachment of each panel only near its physical detachment tick', () => {
  const map: SkylineContent['map'] = JSON.parse(mapJson);
  const {west_tick, east_tick} = map.schedule;
  for (const [tick, x, z] of [[west_tick,-10,-2],[east_tick,10,2]] as const) {
    expect(recentBreakawayMount(map, tick-1)).toBeUndefined();
    expect(recentBreakawayMount(map, tick)).toEqual([x,0,z]);
    expect(recentBreakawayMount(map, tick+12)).toEqual([x,0,z]);
    expect(recentBreakawayMount(map, tick+13)).toBeUndefined();
  }
});

it('plays each fresh panel break once alongside the next warning and drops stale transitions', () => {
  const cues = new MatchCues();
  const source = {pan:-0.7, distance:16};
  const frame = (stage: Parameters<MatchCues['observe']>[1], now: number, epoch = '1') =>
    cues.observe(epoch,stage,[],now,[],{gripTargets:[],stamina:[],breakaway:source});
  expect(frame('west_warning',0)).toEqual({});
  expect(frame('east_warning',50)).toEqual({warning:'east_warning',breakaway:source});
  expect(frame('west_warning',55)).toEqual({});
  expect(frame('east_warning',60)).toEqual({});
  expect(frame('gust_warning',90)).toEqual({warning:'gust_warning',breakaway:source});
  expect(frame('gust',100)).toEqual({warning:'gust'});
  expect(frame('quiet',120,'2')).toEqual({});
  expect(frame('east_warning',140,'2').breakaway).toBeUndefined();
  expect(frame('gust_warning',900,'2').breakaway).toBeUndefined();
  cues.reset();
  expect(frame('east_warning',910,'3')).toEqual({});
});
