import type { Page } from '@playwright/test';
import type { Scene } from 'three';

export function cameraAim(page: Page) {
  return page.evaluate(() => {
    const scene = window.skylineScenes.map(ref => ref.deref()).find(s => s?.getObjectByName('deck-1'));
    if (!scene) return null;
    // Intersect the drawn camera's forward ray with the roof at pelvis height.
    // Measuring the actual draw also catches a selector that updates only its label.
    if (!scene.userData.probingAim) {
      scene.userData.probingAim = true;
      const previous = scene.onBeforeRender;
      scene.onBeforeRender = function(this: Scene, ...args) {
        previous.apply(this, args);
        const camera = args[2], matrix = camera.matrixWorld.elements;
        this.userData.aim = { x: camera.position.x + (1 - camera.position.y) * matrix[8] / matrix[9],
          yaw: Math.atan2(matrix[8], matrix[10]) };
      };
    }
    return (scene.userData.aim as {x: number; yaw: number} | undefined) ?? null;
  });
}
