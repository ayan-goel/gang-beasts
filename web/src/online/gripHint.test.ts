import { expect, it } from 'vitest';
import { gripHint } from './gripHint';

it('shows the controls available to each side of a fighter grip', () => {
  const grips: [number,number][] = [[17,20],[0,0]];
  expect(gripHint(0, grips)).toMatch(/lift.*throw/);
  expect(gripHint(1, grips)).toMatch(/grab back with a free hand/);
  grips[1] = [0,3];
  expect(gripHint(1, grips)).toMatch(/other fighter’s grip attached/);
  expect(gripHint(0, grips)).toMatch(/other fighter’s grip attached/);
});

it('resolves incoming grips by roster index and accounts for occupied hands', () => {
  const grips: [number, number][] = Array.from({length: 8}, () => [0,0]);
  grips[0] = [0,115];
  grips[7] = [65537,65538];
  expect(gripHint(7, grips)).toMatch(/Free a hand/);
  expect(gripHint(1, grips)).toBe('');
  grips[0] = [0,0];
  expect(gripHint(7, grips)).toBe('');
});
