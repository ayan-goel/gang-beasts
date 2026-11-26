import type { Page } from '@playwright/test';
import type { SkinnedMesh } from 'three';

export type JumpDraw = { keyMs: number; riseMs?: number; sendMs?: number; confirmedRiseMs?: number; startY: number; riseY?: number };
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
    let sequence: number | undefined;
    let lastServerY: number | undefined, startServerY: number | undefined;
    const tracked = new WeakSet<WebSocket>();
    function snapshot(event: MessageEvent) {
      if (!(event.data instanceof ArrayBuffer) || event.data.byteLength < 112) return;
      const view = new DataView(event.data);
      if (view.getUint8(0) !== 2) return;
      const flags = view.getUint16(2,true);
      // The host owns slot zero. Its pelvis is the third body in both full snapshot layouts.
      const pelvisAt = 20 + 2 * (flags & 4 ? 23 : 32);
      if (view.getUint32(pelvisAt,true) !== 3) return;
      lastServerY = view.getFloat32(pelvisAt + 8,true);
      if (active && active.confirmedRiseMs === undefined && sequence !== undefined && startServerY !== undefined &&
          flags & 2 && ((view.getUint32(8,true) - sequence) >>> 0) < 0x80000000 && lastServerY >= startServerY + .03) {
        active.confirmedRiseMs = performance.now() - active.keyMs;
      }
    }
    const send = WebSocket.prototype.send;
    WebSocket.prototype.send = function(data) {
      if (!tracked.has(this)) { this.addEventListener('message',snapshot); tracked.add(this); }
      if (active && active.sendMs === undefined && data instanceof Uint8Array && data.length === 18 &&
          data[0] === 1 && (data[14]! & 1)) {
        active.sendMs = performance.now() - active.keyMs;
        startServerY = lastServerY;
        sequence = new DataView(data.buffer,data.byteOffset).getUint32(2,true);
      }
      send.call(this,data);
    };
  });
}
