import type { Playground } from './Playground';

export const maximumSettlingTicks = 240;
export const walkingTicks = 240;
export const approachTicks = walkingTicks + 180;

export function validStairFrame(settle: number, elapsed: number): boolean {
  return Number.isInteger(settle) && settle >= 0 && settle <= maximumSettlingTicks
    && Number.isInteger(elapsed) && elapsed >= 0 && elapsed <= approachTicks;
}

export function inspectStairFrame(playground: Playground, settle: number, elapsed: number): void {
  const { simulation } = playground;
  const movement = simulation.rig?.movement;
  if (!movement || !validStairFrame(settle, elapsed)) throw new Error('Invalid stair frame');

  playground.setPaused(true);
  playground.reset();
  try {
    for (let tick = 0; tick < settle + elapsed; tick++) {
      if (tick === settle) movement.setInput(0, -1, false);
      if (tick === settle + walkingTicks) movement.setInput(0, 0, false);
      simulation.step();
    }
  } finally {
    // Manual playback resumes from the selected pose with the scripted walk released.
    movement.setInput(0, 0, false);
    playground.resetFrameTime();
  }
}
