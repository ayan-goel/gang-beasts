import type { Playground } from './Playground';

export const maximumSettlingTicks = 240;
export const defaultWalkTicks = 240;
export const maximumWalkTicks = 660;
export const stopTicks = 180;

export function validStairFrame(settle: number, elapsed: number, walk = defaultWalkTicks): boolean {
  return Number.isInteger(settle) && settle >= 0 && settle <= maximumSettlingTicks
    && Number.isInteger(walk) && walk >= 0 && walk <= maximumWalkTicks
    && Number.isInteger(elapsed) && elapsed >= 0 && elapsed <= walk + stopTicks;
}

export function inspectStairFrame(playground: Playground, settle: number, elapsed: number, walk = defaultWalkTicks): void {
  const { simulation } = playground;
  const movement = simulation.rig?.movement;
  if (!movement || !validStairFrame(settle, elapsed, walk)) throw new Error('Invalid stair frame');

  playground.setPaused(true);
  playground.reset();
  try {
    for (let tick = 0; tick < settle + elapsed; tick++) {
      if (walk > 0 && tick === settle) movement.setInput(0, -1, false);
      if (tick === settle + walk) movement.setInput(0, 0, false);
      simulation.step();
    }
  } finally {
    // Manual playback resumes from the selected pose with the scripted walk released.
    movement.setInput(0, 0, false);
    playground.resetFrameTime();
  }
}
