import { expect, test, vi } from 'vitest';
import { MovementInput } from './MovementInput';

test('keyboard and pointer intent normalize, combine, and release independently', () => {
  const sent: unknown[] = [];
  const input = new MovementInput({ setInput: (...args) => sent.push(args), setJump: vi.fn(), clearJumpInput: vi.fn() }, () => true);
  input.key('KeyW', true); input.key('KeyD', true); input.key('ShiftLeft', true);
  expect(input.command[0]).toBeCloseTo(Math.SQRT1_2, 12);
  expect(input.command[1]).toBeCloseTo(-Math.SQRT1_2, 12);
  expect(input.command[2]).toBe(true);
  input.key('KeyD', false); expect(sent.at(-1)).toEqual([0,-1,true]);
  input.clear(); expect(sent.at(-1)).toEqual([0,0,false]);
  input.pointer(1, [1,0]); input.pointer(2, [0,1]); input.releasePointer(1);
  expect(sent.at(-1)).toEqual([0,1,false]);
  input.releasePointer(2); expect(sent.at(-1)).toEqual([0,0,false]);
});

test('unfocused lifecycle clears controls and disabled input cannot resume a held key', () => {
  let allowed = true;
  const sent: unknown[] = [];
  const input = new MovementInput({ setInput: (...args) => sent.push(args), setJump: vi.fn(), clearJumpInput: vi.fn() }, () => allowed);
  input.key('KeyW',true); allowed = false; input.clear();
  expect(input.key('KeyW',true)).toBe(false);
  expect(sent.at(-1)).toEqual([0,0,false]);
  allowed = true; input.key('ShiftRight',true);
  expect(sent.at(-1)).toEqual([0,0,false]);
  expect(input.key('KeyQ',true)).toBe(false);
});

test('W and ArrowUp share directional weight but retain independent releases', () => {
  const input = new MovementInput({ setInput: vi.fn(), setJump: vi.fn(), clearJumpInput: vi.fn() }, () => true);
  input.key('KeyW', true); input.key('KeyD', true);
  const diagonal = input.command;
  input.key('ArrowUp', true);
  expect(input.command).toEqual(diagonal);
  input.key('KeyW', false);
  expect(input.command).toEqual(diagonal);
  input.key('ArrowUp', false);
  expect(input.command).toEqual([1, 0, false]);
});

test('a held Space key submits one edge, release retains taps, and blur cancels pending input', () => {
  let allowed = true;
  const port = { setInput: vi.fn(), setJump: vi.fn(), clearJumpInput: vi.fn() };
  const input = new MovementInput(port, () => allowed);
  input.key('Space', true);
  expect(input.key('Space', true, true)).toBe(true);
  input.key('Space', true);
  input.tapJump();
  input.key('Space', false);
  input.tapJump();
  expect(port.setJump.mock.calls).toEqual([[true], [false], [true], [false]]);
  input.clear();
  expect(input.key('Space', true, true)).toBe(false);
  allowed = false;
  expect(input.key('Space', true)).toBe(false);
  input.tapJump();
  expect(port.setJump).toHaveBeenCalledTimes(4);
  expect(port.clearJumpInput).toHaveBeenCalledOnce();
});
