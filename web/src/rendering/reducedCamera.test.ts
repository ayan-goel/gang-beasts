import { PerspectiveCamera, Vector3 } from 'three';
import { expect, it } from 'vitest';
import { GroupCamera } from './GroupCamera';

function pose(x: number, y: number) {
  const state = new Float32Array(15 * 14);
  for (let i = 0; i < 15; i++) state.set([x, y, 0], i * 14);
  return state;
}

it('holds framing through small movements and continues to fit fighters who separate or fall', () => {
  for (const aspect of [.55, 1.8]) {
    const camera = new PerspectiveCamera(45, aspect, .1, 150);
    const group = new GroupCamera([], Array(15).fill(.3));
    group.reducedMotion = true;
    group.update(camera, [pose(-3, 1), pose(3, 1)], false, 1/60);
    const start = camera.position.clone();
    for (let i = 0; i < 120; i++) {
      const sway = .18 * Math.sin(i/8);
      group.update(camera, [pose(-3 + sway, 1), pose(3 + sway, 1 + Math.abs(sway))], false, 1/60);
      expect(camera.position.distanceTo(start)).toBeLessThan(.00001);
    }
    const players = [pose(2, 1), pose(11, -6)];
    for (let i = 0; i < 120; i++) {
      group.update(camera, players, false, 1/60);
      camera.updateMatrixWorld();
      for (const player of players) {
        const point = new Vector3().fromArray(player).project(camera);
        expect(Math.max(Math.abs(point.x), Math.abs(point.y))).toBeLessThan(.7);
      }
    }
    expect(camera.position.distanceTo(start)).toBeGreaterThan(1);
  }
});

it('applies a requested reduced-motion orbit on one frame and uses that heading for input', () => {
  const group = new GroupCamera([]);
  const camera = new PerspectiveCamera(45, 1, .1, 150);
  const start = group.movement(0,-1);
  group.reducedMotion = true;
  group.turn(1);
  expect(group.update(camera, [], true, 1/60)).toBe(true);
  const [x,z] = group.movement(0,-1);
  expect(x).toBeCloseTo((start[0]+start[1])*Math.SQRT1_2, 6);
  expect(z).toBeCloseTo((start[1]-start[0])*Math.SQRT1_2, 6);
  expect(group.update(camera, [], true, 1/60)).toBe(false);
});
