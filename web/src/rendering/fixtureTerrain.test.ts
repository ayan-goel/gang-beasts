import { expect, it } from 'vitest';
import { Box3, Vector3 } from 'three';
import { fixtureTerrain } from './fixtureTerrain';

it('places rendered corners at the physical cuboid limits in all three axes', () => {
  const group = fixtureTerrain(new Float32Array([2, 1, -3, 4, 0.5, 1, -1, 2, -5, 2, 1, 3]));
  expect(group.children).toHaveLength(2);
  const bounds = group.children.map(mesh => new Box3().setFromObject(mesh));
  expect(bounds[0]!.min).toEqual(new Vector3(-2, 0.5, -4));
  expect(bounds[0]!.max).toEqual(new Vector3(6, 1.5, -2));
  expect(bounds[1]!.min).toEqual(new Vector3(-3, 1, -8));
  expect(bounds[1]!.max).toEqual(new Vector3(1, 3, -2));
});
