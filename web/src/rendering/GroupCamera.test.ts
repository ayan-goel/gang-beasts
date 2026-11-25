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
