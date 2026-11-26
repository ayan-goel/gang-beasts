import { readFileSync } from 'node:fs';
import type { Page } from '@playwright/test';
import type { RigDefinition } from '../../src/game/rigDefinition';
import { decodeSnapshot, type BodyPose } from '../../src/net/snapshot';

const rig: RigDefinition = JSON.parse(readFileSync(new URL('../../../config/character.json', import.meta.url), 'utf8'));
const mass = rig.bodies.reduce((sum, part) => sum + part.mass, 0);

function bottom(pose: BodyPose, shape: RigDefinition['bodies'][number]['shape']) {
  const [x,y,z,w] = pose.rotation;
  const vertical = [2*(x*y+w*z), 1-2*(x*x+z*z), 2*(y*z-w*x)].map(Math.abs);
  let extent: number;
  if (shape.kind === 'sphere') extent = shape.radius;
  else if (shape.kind === 'capsule') extent = shape.half_height * vertical[1]! + shape.radius;
  else extent = shape.half_extents.reduce((sum, half, axis) => sum + half*vertical[axis]!, shape.border_radius);
  return pose.position[1] - extent;
}

export interface LiftSample {
  tick: number;
  grips: [number, number];
  counter: [number, number];
  center: [number, number, number];
  clearance: number;
}

export function measureFighter(poseAt: (index: number) => BodyPose) {
  const center: [number, number, number] = [0,0,0];
  let clearance = Infinity;
  for (const [index, part] of rig.bodies.entries()) {
    const pose = poseAt(index);
    for (const axis of [0,1,2] as const) center[axis] += part.mass * pose.position[axis] / mass;
    clearance = Math.min(clearance, bottom(pose, part.shape));
  }
  return {center, clearance};
}

export function observeLift(page: Page) {
  const samples: LiftSample[] = [];
  page.on('websocket', socket => socket.on('framereceived', ({payload}) => {
    if (!Buffer.isBuffer(payload) || payload[0] !== 2) return;
    const snapshot = decodeSnapshot(payload);
    const body = measureFighter(index => snapshot.bodies.get(17 + index)!);
    samples.push({tick: snapshot.tick, grips: snapshot.fighters[0]!.grips,
      counter: snapshot.fighters[1]!.grips, ...body});
  }));
  return samples;
}
