import { expect, it } from 'vitest';
import { fixedLedge } from './ledgeHint';

it('offers a pull on the main roof and each walkway until it falls', () => {
  for (const phase of ['quiet','west_warning','east_warning','gust_warning','gust'] as const) {
    expect(fixedLedge(1, phase)).toBe(true);
    expect(fixedLedge(8, phase)).toBe(['quiet','west_warning'].includes(phase));
    expect(fixedLedge(9, phase)).toBe(['quiet','west_warning','east_warning'].includes(phase));
    for (const id of [undefined, 0, 2, 3, 4, 5, 6, 7]) expect(fixedLedge(id, phase)).toBe(false);
  }
});
