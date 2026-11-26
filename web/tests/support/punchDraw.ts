import type { Page } from '@playwright/test';
import type { SkinnedMesh } from 'three';

export type PunchTrial = {
  hand: 0 | 1; source: 'keyboard' | 'mouse'; inputMs: number;
  sendMs?: number; acceptedMs?: number; drawMs?: number;
};

declare global {
  interface Window {
    punchTrials: PunchTrial[];
    punchReady: boolean;
  }
}

export async function observePunchDraws(page: Page) {
  await page.evaluate(() => {
    const scene = window.skylineScenes.map(ref => ref.deref()).find(s => s?.getObjectByName('deck-1'));
    const mesh = scene?.getObjectByName('RoofCrew') as SkinnedMesh | undefined;
    const chest = mesh?.skeleton.getBoneByName('chest');
    if (!mesh || !chest) throw new Error('Missing host fighter');
    const hands = ['left_hand','right_hand'].map(name => mesh.skeleton.getBoneByName(name)!);
    const positions = hands.map(bone => bone.position.clone());
    const quiet = positions.map(point => point.clone());
    const before = positions[0]!.clone(), inverse = chest.matrixWorld.clone();
    let quietSince = performance.now(), idle = false, sequence: number | undefined;
    let active: PunchTrial | undefined;
    window.punchTrials = [];
    const draw = mesh.onAfterRender;
    mesh.onAfterRender = function(...args) {
      draw.apply(this,args);
      const now = performance.now();
      inverse.copy(chest.matrixWorld).invert();
      hands.forEach((hand,i) => {
        const point = positions[i]!.setFromMatrixPosition(hand.matrixWorld).applyMatrix4(inverse);
        if (point.distanceTo(quiet[i]!) > .006) { quiet[i]!.copy(point); quietSince = now; }
        if (active?.hand === i && active.drawMs === undefined && point.distanceTo(before) >= .03) {
          active.drawMs = now - active.inputMs;
        }
      });
      window.punchReady = idle && now - quietSince >= 250 && (!active || now - active.inputMs >= 1000);
    };
    function start(hand: PunchTrial['hand'], source: PunchTrial['source'], time: number) {
      active = {hand,source,inputMs:time};
      before.copy(positions[hand]!);
      sequence = undefined;
      window.punchTrials.push(active);
      window.punchReady = false;
    }
    const canvas = document.querySelector('canvas')!;
    canvas.addEventListener('keydown', e => {
      if (e.isTrusted && !e.repeat && (e.code === 'KeyJ' || e.code === 'KeyK')) start(e.code === 'KeyK' ? 1 : 0,'keyboard',e.timeStamp);
    },true);
    canvas.addEventListener('mousedown', e => {
      if (e.isTrusted && (e.button === 0 || e.button === 2)) start(e.button === 2 ? 1 : 0,'mouse',e.timeStamp);
    },true);

    function snapshot(e: MessageEvent) {
      if (!(e.data instanceof ArrayBuffer) || e.data.byteLength < 20) return;
      const v = new DataView(e.data);
      if (v.getUint8(0) !== 2) return;
      const flags = v.getUint16(2,true), count = v.getUint16(16,true);
      const host = 20 + (flags & 4 ? 23 : 32) * count + 24;
      const left = v.getUint8(host+3), right = v.getUint8(host+4);
      idle = left === 0 && right === 0;
      if (!active || active.acceptedMs !== undefined || sequence === undefined || !(flags & 2)) return;
      const phase = active.hand === 0 ? left : right;
      if ((phase === 1 || phase === 2) && ((v.getUint32(8,true) - sequence) >>> 0) < 0x80000000) {
        active.acceptedMs = performance.now() - active.inputMs;
      }
    }
    const seen = new WeakSet<WebSocket>(), send = WebSocket.prototype.send;
    WebSocket.prototype.send = function(data) {
      if (!seen.has(this)) { this.addEventListener('message',snapshot); seen.add(this); }
      if (active && active.sendMs === undefined && data instanceof Uint8Array && data.length === 18 &&
          data[0] === 1 && (data[14]! & (4 << active.hand))) {
        sequence = new DataView(data.buffer,data.byteOffset).getUint32(2,true);
        active.sendMs = performance.now() - active.inputMs;
      }
      send.call(this,data);
    };
  });
}
