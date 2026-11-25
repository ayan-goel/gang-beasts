import { expect, test } from 'vitest';
import { inspectMovement, type MovementPort } from './movementInspection';

test('stopping moment is measured separately from yaw and acceleration', () => {
  const movement = {
    state: () => new Float32Array([1,0,0,0,0,0,1,1,0,2,1]),
    stoppingMoment: () => new Float32Array([3,4,0]),
    setInput: () => {},
  };
  expect(inspectMovement(movement)).toMatchObject({acceleration:1,turnTorque:2,stoppingMoment:5});
  for (const values of [[1,2], [NaN,0,0], [Infinity,0,0]]) {
    expect(() => inspectMovement({...movement,stoppingMoment:()=>new Float32Array(values)} as MovementPort)).toThrow('Invalid movement diagnostics');
  }
});
