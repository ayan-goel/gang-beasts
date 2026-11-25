import { expect, it } from 'vitest';
import { InputSampler } from './InputSampler';
import { buttons } from './inputFrame';

it('preserves taps and distinct repeated edges between 60 Hz samples', () => {
  const input = new InputSampler(0xffffffff);
  input.setButton('leftHand', true);
  input.setButton('leftHand', true);
  input.setButton('leftHand', false);
  input.setButton('leftHand', true);
  input.setButton('jump', true);
  const bytes = input.sample(25).map(b => new DataView(b.buffer));
  expect(bytes.map(v => [v.getUint32(2, true), v.getUint16(12, true), v.getUint16(14, true), v.getUint16(16, true)])).toEqual([
    [0xffffffff, 0, buttons.leftHand, buttons.leftHand],
    [0, buttons.leftHand | buttons.jump, buttons.leftHand | buttons.jump, 0],
  ]);
  expect(bytes.every(v => v.getUint32(6, true) === 25)).toBe(true);
  expect(input.sample(26)[0]!.slice(12)).toEqual(Uint8Array.of(5, 0, 0, 0, 0, 0));
});

it('neutralizing cancels unsent taps and releases previously transmitted holds', () => {
  const input = new InputSampler();
  input.setMovement(-0.5, 0);
  input.setButton('rightHand', true);
  input.sample(1);
  input.setButton('jump', true);
  input.setButton('jump', false);
  input.neutralize();
  expect(input.sample(2)[0]!.slice(10)).toEqual(Uint8Array.of(0, 0, 0, 0, 0, 0, buttons.rightHand, 0));
});

it('bounds a burst and recovers with a neutral record', () => {
  const input = new InputSampler();
  input.setButton('lift', true);
  input.sample(1);
  expect(() => {
    for (let i = 0; i < 12; i++) input.setButton('jump', i % 2 === 0);
  }).toThrow(/burst/);
  const recovered = input.sample(2);
  expect(recovered).toHaveLength(1);
  expect(recovered[0]!.slice(10)).toEqual(Uint8Array.of(0, 0, 0, 0, 0, 0, buttons.lift, 0));
});
