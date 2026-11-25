import { expect, test } from 'vitest';
import { inspectMovement, type MovementPort } from './movementInspection';

test('support and stopping moments are measured separately from yaw and acceleration', () => {
  const movement = {
    state: () => new Float32Array([1,0,0,0,0,0,1,1,0,2,1]),
    stoppingMoment: () => new Float32Array([3,4,0]),
    supportMoment: () => new Float32Array([0,5,12]),
    setJump: () => {}, clearJumpInput: () => {},
    setInput: () => {},
  };
  expect(inspectMovement(movement)).toMatchObject({acceleration:1,turnTorque:2,stoppingMoment:5,supportMoment:13});
  for (const values of [[1,2], [NaN,0,0], [Infinity,0,0]]) {
    for (const field of ['stoppingMoment','supportMoment']) {
      expect(() => inspectMovement({...movement,[field]:()=>new Float32Array(values)} as MovementPort)).toThrow('Invalid movement diagnostics');
    }
  }
});
