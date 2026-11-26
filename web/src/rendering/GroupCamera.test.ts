import { expect, test } from 'vitest';
import { PerspectiveCamera, Vector3 } from 'three';
import { GroupCamera } from './GroupCamera';

test('framing keeps separated and falling fighters visible across portrait and landscape screens', () => {
  for (const aspect of [0.55, 1, 1.8]) {
    const camera = new PerspectiveCamera(45, aspect, 0.1, 150);
    const group = new GroupCamera();
    for (const shift of [0, 3, 8, -9]) {
      const players = [-1, 1].map(side => {
        const state = new Float32Array(210);
        for (let i = 0; i < 15; i++) {
          state[i * 14] = side * shift + 0.3; state[i * 14 + 1] = i / 14 + (side === -1 ? -7 : 0); state[i * 14 + 2] = side * 2;
        }
        return state;
      });
      group.update(camera, players, false, 1 / 60); camera.updateMatrixWorld();
      for (const player of players) for (let i = 0; i < player.length; i += 14) {
        const point = new Vector3().fromArray(player, i).project(camera);
        expect(Math.abs(point.x)).toBeLessThan(0.85);
        expect(Math.abs(point.y)).toBeLessThan(0.80);
        expect(Math.abs(point.z)).toBeLessThan(1);
      }
    }
  }
});

test('held movement follows the displayed orbit through eight smooth turns independently of other views', () => {
  const group = new GroupCamera(), other = new GroupCamera();
  const camera = new PerspectiveCamera(45, 1, 0.1, 150);
  const start = other.movement(0,-1);
  group.update(camera, [], true, 0);
  const direction = new Vector3();
  for (let turn = 1; turn <= 8; turn++) {
    let previous = group.movement(0,-1);
    group.turn(1);
    expect(group.movement(0,-1)).toEqual(previous);
    for (let frame = 0; frame < 120; frame++) {
      group.update(camera, [], true, 1/60);
      camera.getWorldDirection(direction);
      direction.y = 0;
      direction.normalize();
      const next = group.movement(0,-1);
      expect(next[0]).toBeCloseTo(direction.x, 6);
      expect(next[1]).toBeCloseTo(direction.z, 6);
      expect(Math.hypot(next[0]-previous[0], next[1]-previous[1])).toBeLessThan(.15);
      previous = next;
    }
    const angle = turn * Math.PI / 4;
    expect(previous[0]).toBeCloseTo(start[0]*Math.cos(angle)+start[1]*Math.sin(angle), 4);
    expect(previous[1]).toBeCloseTo(start[1]*Math.cos(angle)-start[0]*Math.sin(angle), 4);
  }
  group.turn(-1);
  for (let i=0; i<120; i++) group.update(camera, [], true, 1/60);
  expect(group.movement(0,-1)[0]).toBeGreaterThan(start[0]);
  expect(other.movement(0,-1)).toEqual(start);
});
