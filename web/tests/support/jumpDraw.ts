import type { Page } from '@playwright/test';
import type { SkinnedMesh } from 'three';

export type JumpDraw = { keyMs: number; riseMs?: number; sendMs?: number; startY: number; riseY?: number };
declare global {
  interface Window {
    jumpDraws: JumpDraw[];
    readyForJump: boolean;
  }
}

export async function observeLocalJumpDraws(page: Page) {
  await page.evaluate(() => {
    const scene = window.skylineScenes.map(ref => ref.deref()).find(s => s?.getObjectByName('deck-1'));
    const mesh = scene?.getObjectByName('RoofCrew') as SkinnedMesh | undefined;
    const pelvis = mesh?.skeleton.getBoneByName('pelvis');
    if (!mesh || !pelvis) throw new Error('Local fighter skin is missing');
    const canvas = document.querySelector('canvas')!;
    const restY = pelvis.matrixWorld.elements[13]!;
    let lastY = restY, steadyAt = performance.now(), active: JumpDraw | undefined;
    window.jumpDraws = [];
    const originalDraw = mesh.onAfterRender;
    mesh.onAfterRender = function(...args) {
      originalDraw.apply(this,args);
      const now = performance.now(), y = pelvis.matrixWorld.elements[13]!;
      if (active && active.riseMs === undefined && y >= active.startY + .03) {
        active.riseMs = now - active.keyMs;
        active.riseY = y;
      }
      if (Math.abs(y - restY) > .06 || Math.abs(y - lastY) > .005) steadyAt = now;
      window.readyForJump = now - steadyAt > 250 && !active;
      lastY = y;
    };
    canvas.addEventListener('keydown', event => {
      if (event.code !== 'Space' || event.repeat || !event.isTrusted) return;
      active = {keyMs:event.timeStamp, startY:lastY};
      window.jumpDraws.push(active);
      window.readyForJump = false;
    },true);
    const send = WebSocket.prototype.send;
    WebSocket.prototype.send = function(data) {
      if (active && active.sendMs === undefined && data instanceof Uint8Array && data.length === 18 &&
          data[0] === 1 && (data[14]! & 1)) active.sendMs = performance.now() - active.keyMs;
      send.call(this,data);
    };
  });
}
