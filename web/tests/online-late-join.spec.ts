import {expect, test, type Browser, type Page} from '@playwright/test';
import {decodeSnapshot} from '../src/net/snapshot';
import {gripMatch} from './support/gripMatch';
import {releaseRoom} from './support/releaseRoom';
import {observeArenaMeshes} from './support/arenaMeshes';
import {cameraAim} from './support/cameraAim';

if (process.platform === 'darwin') test.use({launchOptions:{args:['--use-gl=angle','--use-angle=metal']}});

function observe(page: Page) {
  const view = {sockets:0, inputs:0, tick:0, players:0, x:0, handoffs:[] as number[]};
  page.on('websocket', socket => {
    if (!socket.url().includes('/ws?')) return;
    view.sockets++;
    socket.on('framesent', ({payload}) => {
      if (Buffer.isBuffer(payload) && payload[0] === 1) view.inputs++;
    });
    socket.on('framereceived', ({payload}) => {
      if (!Buffer.isBuffer(payload)) return;
      if (payload[0] === 27 || payload[0] === 34) view.handoffs.push(payload[0]);
      if (payload[0] !== 2) return;
      const frame = decodeSnapshot(payload);
      view.players = frame.fighters.length;
      view.tick = frame.tick;
      if (view.players === 3) view.x = frame.bodies.get(35)!.position[0];
    });
  });
  return view;
}

async function verifyJoin(host: Page, browser: Browser, legacyPeer: boolean) {
  const profile = await browser.newContext();
  const newcomer = await profile.newPage();
  if (!legacyPeer) await newcomer.setViewportSize({width:390, height:800});
  await observeArenaMeshes(newcomer);
  let invite = '';
  try {
    await gripMatch(host, browser, async (guest, start) => {
      const views = [host, guest, newcomer].map(observe);
      if (legacyPeer) await guest.addInitScript(() => {
        window.WebSocket = class extends WebSocket {
          constructor(url: string | URL, protocols?: string | string[]) {
            const path = new URL(url, location.href);
            if (path.pathname === '/ws') path.search = '?resume=1';
            super(path, protocols);
          }
        };
      });
      await start();
      invite = guest.url();
      await newcomer.goto(invite);
      await newcomer.getByLabel('Player name').fill('Gold');
      await newcomer.getByRole('button',{name:'Join room',exact:true}).click();
      await expect(newcomer.getByRole('heading',{name:'Spectating Skyline'})).toBeVisible({timeout:15000});
      const camera = newcomer.getByRole('group', {name:'Spectator camera'});
      await expect(host.getByRole('group', {name:'Spectator camera'})).toHaveCount(0);
      await expect(camera.getByLabel('Camera view')).toHaveText('Group view');
      await camera.getByRole('button', {name:'Next fighter'}).click();
      await expect(camera.getByLabel('Camera view')).toHaveText('Following Mint');
      await expect.poll(async () => (await cameraAim(newcomer))?.x).toBeLessThan(-1);
      await camera.getByRole('button', {name:'Group view'}).click();
      await expect.poll(async () => Math.abs((await cameraAim(newcomer))?.x ?? Infinity)).toBeLessThan(.65);
      await camera.getByRole('button', {name:'Previous fighter'}).click();
      await expect(camera.getByLabel('Camera view')).toHaveText('Following Blue');
      const initialYaw = (await cameraAim(newcomer))!.yaw;
      await newcomer.keyboard.press('KeyX');
      await expect.poll(async () => (await cameraAim(newcomer))?.yaw).toBeGreaterThan(initialYaw + .6);
      await expect.poll(async () => (await cameraAim(newcomer))?.x).toBeGreaterThan(1);
      if (!legacyPeer) {
        const panel = await camera.boundingBox();
        expect(panel!.x + panel!.width).toBeLessThanOrEqual(390);
        await newcomer.getByLabel('Online Skyline arena').screenshot({path:test.info().outputPath('spectator-camera.png')});
      }
      await newcomer.locator('canvas').focus();
      await newcomer.keyboard.press('Tab');
      const board = newcomer.getByRole('region', {name:'Scoreboard',exact:true});
      await expect(board).toBeFocused();
      await expect(board.getByLabel('Blue score')).toHaveText('0');
      await newcomer.keyboard.press('Escape');
      await expect(newcomer.locator('canvas')).toBeFocused();
      await newcomer.keyboard.down('KeyD');
      const waiting = views[2]!.tick;
      await expect.poll(() => views[2]!.tick).toBeGreaterThan(waiting + 8);
      await newcomer.keyboard.up('KeyD');
      expect(views[2]!.inputs).toBe(0);
      expect(views.map(v => v.players)).toEqual([2,2,2]);
      await guest.bringToFront();
      await guest.locator('canvas').focus();
      await guest.keyboard.down('ShiftLeft');
      await guest.keyboard.down('KeyD');
      try { await expect(host.getByLabel('Mint score')).toHaveText('1',{timeout:20000}); }
      finally { await guest.keyboard.up('KeyD'); await guest.keyboard.up('ShiftLeft'); }
      await expect(camera.getByLabel('Camera view')).toHaveText('Following Mint');
      await expect.poll(async () => (await cameraAim(newcomer))?.x).toBeLessThan(-1);
      for (const page of [host,guest,newcomer]) {
        await expect(page.getByLabel('Round number')).toHaveText('Round 2',{timeout:15000});
        await expect(page.getByLabel('Gold score')).toHaveText('0');
        await expect(page.getByLabel('Mint score')).toHaveText('1');
        await expect(page.getByLabel('Round phase')).toHaveText('Playing',{timeout:12000});
      }
      await expect(newcomer.getByRole('heading',{name:'Playing as Gold'})).toBeVisible();
      await newcomer.bringToFront();
      await newcomer.locator('canvas').focus();
      const before = views.map(v => v.x);
      await newcomer.keyboard.down('KeyD');
      try { await expect.poll(() => views.every((v,i) => v.x > before[i]! + 1)).toBe(true); }
      finally { await newcomer.keyboard.up('KeyD'); }
      expect(views[2]!.inputs).toBeGreaterThan(0);
      expect(views.map(v => v.handoffs)).toEqual([[34],[legacyPeer ? 27 : 34],[34]]);
      expect(views.map(v => [v.sockets, v.players])).toEqual([[1,3],[legacyPeer ? 2 : 1,3],[1,3]]);
    });
  } finally {
    try { await releaseRoom(invite,[profile]); } finally { await profile.close(); }
  }
}

test('a late spectator enters the next round with their own fighter on the same socket', async ({page,browser}) => {
  await verifyJoin(page,browser,false);
});

test('a peer using the original handshake receives its new lineup over a fresh socket', async ({page,browser}) => {
  await verifyJoin(page,browser,true);
});
