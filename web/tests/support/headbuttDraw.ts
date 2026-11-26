import type { Page } from '@playwright/test';
import type { SkinnedMesh } from 'three';

export interface HeadbuttTrial { keyMs: number; sentMs?: number; acceptedMs?: number; drawnMs?: number }
declare global {
  interface Window { headbuttTrial?: HeadbuttTrial; headSettled: boolean }
}

export async function observeHeadbutt(page: Page) {
  await page.evaluate(() => {
    const scene = window.skylineScenes.map(ref => ref.deref()).find(s => s?.getObjectByName('deck-1'));
    const mesh = scene?.getObjectByName('RoofCrew') as SkinnedMesh | undefined;
    const head = mesh?.skeleton.getBoneByName('head');
    if (!mesh || !head || head.parent?.name !== 'chest') throw new Error('Missing host head or chest');
    const pose = head.quaternion.clone().setFromRotationMatrix(head.matrix);
    const quiet = pose.clone(), before = pose.clone();
    let quietSince = performance.now(), stamina = 0, sequence: number | undefined;
    const draw = mesh.onAfterRender;
    mesh.onAfterRender = function(...args) {
      draw.apply(this,args);
      const now = performance.now();
      pose.setFromRotationMatrix(head.matrix);
      if (pose.angleTo(quiet) > Math.PI/180) { quiet.copy(pose); quietSince = now; }
      window.headSettled = stamina === 100 && now - quietSince >= 250;
      const trial = window.headbuttTrial;
      if (trial && trial.drawnMs === undefined && pose.angleTo(before) >= 5*Math.PI/180) trial.drawnMs = now - trial.keyMs;
    };
    document.querySelector('canvas')!.addEventListener('keydown', e => {
      if (e.isTrusted && !e.repeat && e.code === 'KeyQ') {
        before.copy(pose);
        window.headbuttTrial = {keyMs:e.timeStamp};
      }
    },true);
    const seen = new WeakSet<WebSocket>();
    const send = WebSocket.prototype.send;
    WebSocket.prototype.send = function(data) {
      if (!seen.has(this)) {
        seen.add(this);
        this.addEventListener('message', (event: MessageEvent) => {
          if (!(event.data instanceof ArrayBuffer) || event.data.byteLength < 20) return;
          const packet = new DataView(event.data);
          if (packet.getUint8(0) !== 2) return;
          const flags = packet.getUint16(2,true);
          const host = 20 + (flags & 4 ? 23 : 32) * packet.getUint16(16,true) + 24;
          stamina = packet.getFloat32(host+12,true);
          const trial = window.headbuttTrial;
          if (trial && trial.acceptedMs === undefined && sequence !== undefined && flags & 2 &&
              ((packet.getUint32(8,true)-sequence) >>> 0) < 0x80000000 && stamina < 99) {
            trial.acceptedMs = performance.now() - trial.keyMs;
          }
        });
      }
      const trial = window.headbuttTrial;
      if (trial && trial.sentMs === undefined && data instanceof Uint8Array && data.length === 18 &&
          data[0] === 1 && (data[14]! & 32)) {
        sequence = new DataView(data.buffer,data.byteOffset).getUint32(2,true);
        trial.sentMs = performance.now() - trial.keyMs;
      }
      send.call(this,data);
    };
  });
}
