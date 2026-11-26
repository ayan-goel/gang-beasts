import { expect, it } from 'vitest';
import { HazardWarnings } from './HazardWarnings';

it('emits each live transition once and starts fresh in a new round', () => {
  const cues = new HazardWarnings();
  expect(cues.observe('1','quiet',0)).toBeUndefined();
  expect(cues.observe('1','west_warning',100)).toBe('west_warning');
  expect(cues.observe('1','west_warning',200)).toBeUndefined();
  expect(cues.observe('1','east_warning',300)).toBe('east_warning');
  expect(cues.observe('1','west_warning',400)).toBeUndefined();
  expect(cues.observe('1','east_warning',500)).toBeUndefined();
  expect(cues.observe('2','quiet',600)).toBeUndefined();
  expect(cues.observe('2','west_warning',700)).toBe('west_warning');
});

it('uses the first state after joining, interruption or a missed publication interval as a baseline', () => {
  const cues = new HazardWarnings();
  expect(cues.observe('1','east_warning',0)).toBeUndefined();
  expect(cues.observe('1','gust_warning',4000)).toBeUndefined();
  expect(cues.observe('1','gust',4100)).toBe('gust');
  cues.reset();
  expect(cues.observe('2','west_warning',4200)).toBeUndefined();
  expect(cues.observe('2','east_warning',4300)).toBe('east_warning');
});
