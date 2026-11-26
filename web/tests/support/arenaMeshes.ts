import type { Page } from '@playwright/test';
import type { Mesh, MeshStandardMaterial, Scene } from 'three';
import {readFileSync} from 'node:fs';
import type {SkylineMap} from '../../src/rendering/SkylineView';

declare global {
  interface Window { skylineScenes: WeakRef<Scene>[] }
}

export async function observeArenaMeshes(page: Page) {
  await page.addInitScript(() => {
    const devtools = new EventTarget();
    window.skylineScenes = [];
    // Three announces scenes before attaching their children. Weak references let a reload or rematch dispose them.
    devtools.addEventListener('observe', event => {
      const object = (event as CustomEvent<Scene>).detail;
      if (object?.isScene) window.skylineScenes.push(new WeakRef(object));
    });
    Object.assign(window, { __THREE_DEVTOOLS__: devtools });
  });
}

const map: SkylineMap = JSON.parse(readFileSync(new URL('../../../assets/maps/skyline/map.json', import.meta.url), 'utf8'));
export const authoredMeshes = map.objects.map(({kind, id, center}) => ({
  name: `${kind}-${id}`, position: center, rotation: [0, 0, 0, 1], visible: true, emissive: 0,
}));

export function arenaMeshes(page: Page) {
  return page.evaluate(names => {
    const scene = window.skylineScenes.slice().reverse().map(ref => ref.deref()).find(s => s?.getObjectByName('deck-1'));
    if (!scene) return [];
    // Compare at centimetre precision to accommodate the compact wire pose and the WebGL float buffer.
    const rounded = (values: number[]) => values.map(v => Math.round(v * 100) / 100 || 0);
    return names.flatMap(name => {
      const mesh = scene.getObjectByName(name) as Mesh | undefined;
      return mesh ? [{ name, position: rounded(mesh.position.toArray()), rotation: rounded(mesh.quaternion.toArray()),
        visible: mesh.visible, emissive: (mesh.material as MeshStandardMaterial).emissive.getHex() }] : [];
    });
  }, authoredMeshes.map(mesh => mesh.name));
}
