import { expect, test } from 'vitest';
import * as THREE from 'three';
import { JointOverlay } from './JointOverlay';

test('target axes compose parent-world and native-relative rotations in the correct order', () => {
  const overlay = new JointOverlay(1);
  const state = new Float32Array(17);
  state.set([0,0,Math.SQRT1_2,Math.SQRT1_2],8); state[15] = 1;
  overlay.update(state, new Float32Array([Math.SQRT1_2,0,0,Math.SQRT1_2]));
  const positions = (overlay.root.children[0] as THREE.LineSegments).geometry.getAttribute('position');
  for (const [vertex, expected] of [[3,[0,0.15,0]], [5,[0,0,0.15]], [7,[0.15,0,0]], [9,[0.15,0,0]]] as const) {
    expect(positions.getX(vertex)).toBeCloseTo(expected[0],6);
    expect(positions.getY(vertex)).toBeCloseTo(expected[1],6);
    expect(positions.getZ(vertex)).toBeCloseTo(expected[2],6);
  }
  expect(()=>overlay.update(state,new Float32Array(3))).toThrow('Invalid joint target buffer');
  overlay.root.traverse(object=>{ if(object instanceof THREE.LineSegments || object instanceof THREE.Points) { object.geometry.dispose(); (object.material as THREE.Material).dispose(); } });
});
