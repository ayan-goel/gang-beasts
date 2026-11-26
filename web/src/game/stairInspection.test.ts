import { expect, test } from 'vitest';
import { inspectStairFrame, validStairFrame } from './stairInspection';
import type { Playground } from './Playground';

function replay(settle: number, elapsed: number, walk: number) {
  const frames: number[] = [];
  let intent = 0;
  const playground = {
    reset:() => {}, resetFrameTime:() => {}, setPaused:() => {},
    simulation:{step:() => frames.push(intent),rig:{movement:{setInput:(_x: number,z: number) => { intent = z; }}}}
  } as unknown as Playground;
  inspectStairFrame(playground,settle,elapsed,walk);
  return {frames,intent};
}

test('the chosen duration controls the release tick and leaves manual playback neutral', () => {
  expect(replay(2,5,3)).toEqual({frames:[0,0,-1,-1,-1,0,0],intent:0});
  expect(replay(1,3,0)).toEqual({frames:[0,0,0,0],intent:0});
  expect(replay(0,1,440)).toEqual({frames:[-1],intent:0});
});

test('the replay can show recovery and stopping while bounding the main-thread solve', () => {
  expect(validStairFrame(108,620,440)).toBe(true);
  expect(validStairFrame(108,621,440)).toBe(false);
  expect(validStairFrame(0,420)).toBe(true);
  expect(validStairFrame(0,421)).toBe(false);
  for (const walk of [-1,2.5,661,NaN]) expect(validStairFrame(25,0,walk)).toBe(false);
});
