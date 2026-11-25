import { describe, expect, it } from 'vitest';
import { Vector3 } from 'three';
import { geometryForBody } from './RagdollView';

describe('physical mesh geometry', () => {
  it('preserves Rapier capsule segment length and rounded-box border dimensions', () => {
    const cases = [
      { shape: { kind: 'capsule' as const, radius: 0.18, half_height: 0.12 }, size: [0.36, 0.6, 0.36] },
      { shape: { kind: 'rounded_box' as const, half_extents: [0.08, 0.05, 0.14] as [number, number, number], border_radius: 0.02 }, size: [0.2, 0.14, 0.32] },
    ];
    for (const { shape, size } of cases) {
      const geometry = geometryForBody(shape);
      geometry.computeBoundingBox();
      const actual = geometry.boundingBox!.getSize(new Vector3()).toArray();
      actual.forEach((value, axis) => expect(value).toBeCloseTo(size[axis]!, 6));
      geometry.dispose();
    }
  });
});
