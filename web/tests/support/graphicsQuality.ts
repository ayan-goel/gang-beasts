import type { Page } from '@playwright/test';
import type { DirectionalLight } from 'three';

export function graphicsQuality(page: Page) {
  return page.evaluate(() => {
    const scene = window.skylineScenes.map(ref => ref.deref()).find(s => s?.getObjectByName('deck-1'));
    const light = scene?.children.find(s => s.type === 'DirectionalLight') as DirectionalLight | undefined;
    const canvas = document.querySelector('canvas');
    if (!canvas || !light?.shadow.map) return null;
    return { ratio: Number((canvas.width / canvas.clientWidth).toFixed(2)), shadow: light.shadow.map.width };
  });
}
