import { expect, test } from 'vitest';
import { ArenaWater } from './Water';

test('foam tracks a moving waterline and ripples follow only swimmers still in the round', () => {
  const water = new ArenaWater(-.6, '#27788e', '#d1dfed', [
    {id:1, kind:'deck', center:[0,-.3,0], half:[2,.3,1]},
  ]);
  const pose = new Float32Array(15*14);
  pose.set([4,-.5,2],28);
  const halfTurn = Math.PI/4;
  const props = new Float32Array([3,-.3,4,0,Math.sin(halfTurn),0,Math.cos(halfTurn),1]);
  const uniforms = water.mesh.material.uniforms;
  water.update(2,[pose],[true],props);
  expect(uniforms.shores!.value[0].toArray()).toEqual([3,4,2,1]);
  expect(uniforms.bearings!.value[0].x).toBeCloseTo(0);
  expect(uniforms.bearings!.value[0].y).toBeCloseTo(-1);
  expect(uniforms.swimmers!.value[0].z).toBeGreaterThan(.8);
  props[1] = -3;
  water.update(3,[pose],[false],props);
  expect(uniforms.shores!.value[0].z).toBeLessThan(0);
  expect(uniforms.swimmers!.value[0].z).toBe(0);
  water.mesh.geometry.dispose();
  water.mesh.material.dispose();
});
