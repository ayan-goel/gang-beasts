import { BasicShadowMap, Mesh, PCFSoftShadowMap, type DirectionalLight, type Scene, type WebGLRenderer } from 'three';
import type { skylineQuality } from './skylineQuality';

export function applySkylineQuality(renderer: WebGLRenderer, scene: Scene, sun: DirectionalLight, quality: ReturnType<typeof skylineQuality>) {
  if (renderer.getPixelRatio() !== quality.pixelRatio) renderer.setPixelRatio(quality.pixelRatio);
  const filter = quality.basic ? BasicShadowMap : PCFSoftShadowMap;
  if (renderer.shadowMap.type !== filter) {
    renderer.shadowMap.type = filter;
    // Three selects the shadow filter when compiling each material's shader.
    scene.traverse(object => {
      if (object instanceof Mesh) {
        for (const material of Array.isArray(object.material) ? object.material : [object.material]) material.needsUpdate = true;
      }
    });
  }
  if (sun.shadow.mapSize.x !== quality.shadowSize) {
    // A size change needs a fresh render target. Release its GPU allocation before the next draw.
    sun.shadow.map?.dispose();
    sun.shadow.map = null;
    sun.shadow.mapSize.setScalar(quality.shadowSize);
  }
}
