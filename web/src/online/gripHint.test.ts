import { expect, it } from 'vitest';
import { gripHint } from './gripHint';

it('shows the controls available to each side of a fighter grip', () => {
  const grips: [number,number][] = [[17,20],[0,0]];
  expect(gripHint(0, grips, 'quiet')).toMatch(/lift.*throw/);
  expect(gripHint(1, grips, 'quiet')).toMatch(/grab back with a free hand/);
  grips[1] = [0,3];
  expect(gripHint(1, grips, 'quiet')).toMatch(/other fighter’s grip attached/);
  expect(gripHint(0, grips, 'quiet')).toMatch(/other fighter’s grip attached/);
});

it('resolves incoming grips by roster index and accounts for occupied hands', () => {
  const grips: [number, number][] = Array.from({length: 8}, () => [0,0]);
  grips[0] = [0,115];
  grips[7] = [65538,65539];
  expect(gripHint(7, grips, 'quiet')).toMatch(/Free a hand/);
  expect(gripHint(1, grips, 'quiet')).toBe('');
  grips[0] = [0,0];
  expect(gripHint(7, grips, 'quiet')).toBe('');
});

it('prioritizes a fixed ledge during a grapple and removes the prompt on detachment', () => {
  const grips: [number,number][] = [[65544,17],[3,0]];
  expect(gripHint(0, grips, 'west_warning')).toBe('Hold Space to pull up');
  expect(gripHint(0, grips, 'east_warning')).toMatch(/other fighter’s grip attached/);
  expect(gripHint(1, grips, 'west_warning')).toMatch(/other fighter’s grip attached/);
});
