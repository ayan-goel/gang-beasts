import { Quaternion } from 'three';
import { groupOf, type PoseSample } from './SnapshotTimeline';

const rotation = [0,0,0,1];

export function writePose(sample: PoseSample, id: number, output: Float32Array, offset: number): boolean {
  const exact = sample.state.bodies.get(id);
  if (!exact) return false;
  const a = sample.from.bodies.get(id), b = sample.to.bodies.get(id);
  if (!a || !b || !sample.factor || sample.cuts.has(groupOf(id))) {
    output.set(exact.position, offset);
    output.set(exact.rotation, offset + 3);
    return true;
  }
  for (let axis = 0; axis < 3; axis++) {
    const start = a.position[axis]!;
    output[offset + axis] = start + sample.factor * (b.position[axis]! - start);
  }
  if (sample.predicted) output.set(b.rotation, offset + 3);
  else {
    Quaternion.slerpFlat(rotation, 0, a.rotation, 0, b.rotation, 0, sample.factor);
    output.set(rotation, offset + 3);
  }
  return true;
}
