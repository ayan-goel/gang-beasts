import type { Page } from '@playwright/test';

export function ventCamera(page: Page, owner: number | null) {
  return page.evaluate(slot => {
    const scene = window.skylineScenes.map(ref => ref.deref()).find(s => s?.getObjectByName('vent-3'));
    if (!scene) return null;
    if (!scene.userData.measuringVent) {
      scene.userData.measuringVent = true;
      const before = scene.onBeforeRender;
      const hips = scene.getObjectsByProperty('name','pelvis');
      const vent = scene.getObjectByName('vent-3')!;
      let anchorZ = 0, steadyFrom = performance.now();
      scene.onBeforeRender = function(...args) {
        before.apply(this,args);
        if (hips.length !== 2) return;
        const camera = args[2], matrix = camera.matrixWorld.elements;
        let z = 0, y = 0;
        for (const [i, hip] of hips.entries()) {
          const weight = slot === i ? 1.35 : 1;
          z += weight * hip.matrixWorld.elements[14]!;
          y += weight * hip.matrixWorld.elements[13]!;
        }
        z /= slot === null ? 2 : 2.35;
        y = Math.max(.8,y/(slot === null ? 2 : 2.35));
        const face = vent.position.z - .7;
        const atFace = camera.position.y+(face-camera.position.z)*matrix[9]/matrix[10];
        const nominal = y+(face-z)*.85;
        const now = performance.now();
        if (Math.abs(z-anchorZ) > .07 || z < 4.4) {
          anchorZ = z;
          steadyFrom = now;
        }
        scene.userData.ventReading = {z,atFace,nominal,steadyMs:now-steadyFrom,pitch:Math.asin(matrix[9]),yaw:Math.atan2(matrix[8],matrix[10])};
      };
    }
    return (scene.userData.ventReading as {steadyMs:number;z:number;atFace:number;nominal:number;pitch:number;yaw:number} | undefined) ?? null;
  },owner);
}
