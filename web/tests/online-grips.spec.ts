import { expect, test, type Page } from '@playwright/test';
import { readFileSync } from 'node:fs';
import type { RigDefinition } from '../src/game/rigDefinition';
import { decodeSnapshot } from '../src/net/snapshot';
import { approachOpponent, gripMatch } from './support/gripMatch';

const rig: RigDefinition = JSON.parse(readFileSync(new URL('../../config/character.json',import.meta.url),'utf8'));

// Both renderers share one host. Software rendering during guest reload can starve
// the owner's input past the server's 250 ms stale-input deadline.
if (process.platform === 'darwin') test.use({launchOptions: {args: ['--use-gl=angle', '--use-angle=metal']}});

function observeHands(page: Page, hand: 0 | 1) {
  const record = {target:0, struckWhileHolding:false, released:false};
  page.on('websocket', socket => socket.on('framereceived', ({payload}) => {
    if (!Buffer.isBuffer(payload) || payload[0] !== 2) return;
    const fighter = decodeSnapshot(payload).fighters[0]!;
    const target = fighter.grips[hand];
    if (!record.target && target) record.target = target;
    if (record.target && target === record.target && fighter.hands[1-hand] === 2) record.struckWhileHolding = true;
    if (record.target && target === 0) record.released = true;
  }));
  return record;
}

for (const hand of [0,1] as const) {
  const side = hand === 0 ? 'Left' : 'Right';
  test(side + ' hand keeps its opponent grip while the other punches in both views', async ({page:host,browser}) => {
    await approachOpponent(host,hand);
    const owner = observeHands(host,hand);
    await gripMatch(host,browser,async (guest,start) => {
      const peer = observeHands(guest,hand);
      await start();
      const pages = [host,guest], grab = hand === 0 ? 'KeyJ' : 'KeyK', punch = hand === 0 ? 'KeyK' : 'KeyJ';
      await host.keyboard.down(grab);
      await expect.poll(() => owner.target,{timeout:25_000}).toBeGreaterThan(16);
      const part = rig.bodies[owner.target-17]!.part.replaceAll('_',' ');
      const notice = 'Mint ' + side + ' hand on Blue’s ' + part;
      await Promise.all(pages.map(p => expect(p.getByLabel('Mint grips')).toHaveText(notice)));
      await host.keyboard.press(punch);
      await expect.poll(() => owner.struckWhileHolding && peer.struckWhileHolding).toBe(true);
      expect(peer.target).toBe(owner.target);
      if (hand === 0) {
        await guest.reload();
        await expect(guest.getByLabel('Mint grips')).toHaveText(notice);
        expect(owner.released).toBe(false);
      }
      await host.keyboard.up(grab);
      await expect.poll(() => owner.released && peer.released).toBe(true);
      await Promise.all(pages.map(p => expect(p.getByLabel('Mint grips')).toHaveCount(0)));
    });
  });
}

function observeCrate(page: Page) {
  const state = {travel:0};
  let anchor: number[] | undefined;
  page.on('websocket', socket => socket.on('framereceived', ({payload}) => {
    if (!Buffer.isBuffer(payload) || payload[0] !== 2) return;
    const frame = decodeSnapshot(payload);
    const id = 65536 + 7;
    if (!frame.fighters[0]!.grips.includes(id)) { anchor = undefined; return; }
    const p = frame.bodies.get(id)!.position;
    anchor ??= p;
    state.travel = Math.max(state.travel,Math.hypot(p[0]-anchor[0]!,p[1]-anchor[1]!,p[2]-anchor[2]!));
  }));
  return state;
}

test('both views identify a moving rooftop crate and clear the grip on focus loss', async ({page:host,browser}) => {
  await gripMatch(host,browser,async (guest,start) => {
    const crate = [host,guest].map(observeCrate);
    await start();
    await host.keyboard.down('KeyS');
    await host.keyboard.down('KeyJ');
    await host.keyboard.down('KeyK');
    const labels = [host,guest].map(p => p.getByLabel('Mint grips'));
    await Promise.all(labels.map(l => expect(l).toContainText('hand on crate',{timeout:15_000})));
    await host.keyboard.down('KeyE');
    await expect.poll(() => Math.min(...crate.map(c => c.travel))).toBeGreaterThan(0.1);
    await host.keyboard.up('KeyS');
    await host.getByRole('button',{name:'Mute game audio'}).focus();
    await Promise.all(labels.map(l => expect(l).toHaveCount(0)));
    await host.keyboard.up('KeyJ');
    await host.keyboard.up('KeyK');
    await host.keyboard.up('KeyE');
  });
});
