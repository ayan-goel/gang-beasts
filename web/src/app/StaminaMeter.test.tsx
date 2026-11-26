import { renderToStaticMarkup } from 'react-dom/server';
import { expect, it } from 'vitest';
import { StaminaMeter } from './StaminaMeter';

function meter(points: number, holding = false) {
  return renderToStaticMarkup(<StaminaMeter name="Blue" points={points} holding={holding} />);
}

it('shows full stamina only while holding', () => {
  expect(meter(100)).toBe('');
  expect(meter(100, true)).toContain('value="100"');
  expect(meter(99.99)).toContain('value="99"');
});

it('identifies the fighter and warns at the last quarter through exhaustion', () => {
  expect(meter(25)).toContain('aria-label="Blue stamina"');
  expect(meter(25)).toContain('Low stamina');
  expect(meter(25.01)).not.toContain('Low stamina');
  expect(meter(25.01)).toContain('value="26"');
  expect(meter(0.01)).toContain('value="1"');
  expect(meter(0)).toContain('value="0"');
});
