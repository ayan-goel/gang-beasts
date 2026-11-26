import { expect, test, type Page } from '@playwright/test';
import { encodeInput, quantizeDirection } from '../src/net/inputFrame';
import { decodeSnapshot } from '../src/net/snapshot';
import { observeArenaMeshes } from './support/arenaMeshes';
import { ventCamera } from './support/ventCamera';
import { gripMatch } from './support/gripMatch';

if (process.platform === 'darwin') test.use({launchOptions:{args:['--use-gl=angle','--use-angle=metal']}});

async function walkToVent(page: Page, slot: number) {
  let z = 4.8;
  let last: number[] = [];
  await page.routeWebSocket(url => url.pathname === '/ws', client => {
    const server = client.connectToServer();
    let sequence = 0;
    client.onMessage(packet => {
      if (!Buffer.isBuffer(packet) || packet[0] !== 1) server.send(packet);
    });
    server.onMessage(packet => {
      client.send(packet);
      if (!Buffer.isBuffer(packet) || packet[0] !== 2) return;
      const frame = decodeSnapshot(packet);
      if (frame.phase !== 1 || frame.tick % 6 !== 0) return;
      const [px,,pz] = frame.bodies.get(3+slot*16)!.position;
      last = [px,pz];
      const dx = (slot === 0 ? 2.4 : 3.6)-px, dz = z-pz;
      const length = Math.hypot(dx,dz);
      const axes = length < .12 ? [0,0] as const : quantizeDirection(dx/length,dz/length);
      server.send(Buffer.from(encodeInput({sequence:sequence++,tick:frame.tick,axes,held:0,pressed:0,released:0})));
    });
  });
  return {leave:() => { z = 1.5; }, position:() => last};
}

const normal = Math.atan2(.85,Math.hypot(.18,1));
async function besideVent(page: Page, owner: number | null) {
  await expect.poll(async () => {
    const s = await ventCamera(page,owner);
    return !!s && s.steadyMs > 800 && s.z > 4.4 && s.nominal < 1.25 && s.atFace > 1.20 && s.atFace-s.nominal > .1;
  },{timeout:25_000,intervals:[50,100]}).toBe(true);
  const view = (await ventCamera(page,owner))!;
  expect(view.yaw).toBeCloseTo(Math.atan2(.18,1),4);
  expect(view.pitch).toBeCloseTo(normal,2);
  return view;
}

async function clearOfVent(page: Page, owner: number | null) {
  await expect.poll(async () => {
    const s = await ventCamera(page,owner);
    return !!s && s.z < 2 && Math.abs(s.atFace-s.nominal) < .05;
  },{timeout:12_000}).toBe(true);
  return ventCamera(page,owner);
}

test('two online fighters walk up to the south vent and the drawn camera clears it', async ({page:host,browser}) => {
  await gripMatch(host,browser,async (peer,start) => {
    await peer.setViewportSize({width:390,height:800});
    const players = [host,peer];
    await Promise.all(players.map(observeArenaMeshes));
    const drivers = await Promise.all(players.map(walkToVent));
    await start();
    try {
      const beside = await Promise.all(players.map(besideVent));
      for (const driver of drivers) driver.leave();
      const returned = await Promise.all(players.map(clearOfVent));
      await test.info().attach('camera-approach',{body:JSON.stringify({beside,returned}),contentType:'application/json'});
      for (const player of players) await expect(player.getByLabel('Round phase')).toHaveText('Playing');
    } finally {
      await test.info().attach('last-views',{body:JSON.stringify({positions:drivers.map(d=>d.position()),
        views:await Promise.all(players.map(ventCamera))}),contentType:'application/json'});
    }
  });
});

test('the shared-screen camera clears the same vent during local physical play', async ({page}) => {
  await observeArenaMeshes(page);
  await page.goto('/play/');
  const start = page.getByRole('button',{name:'Start match',exact:true});
  await expect(start).toBeEnabled();
  await page.evaluate(async () => {
    const {LocalDuel} = await import(/* @vite-ignore */ String('/src/generated/wasm/brawler_wasm.js'));
    const step = LocalDuel.prototype.step;
    let goalZ = 4.8;
    window.addEventListener('retreat-from-vent', () => { goalZ = 1.5; },{once:true});
    LocalDuel.prototype.step = function() {
      if (Number(this.tick()) % 6 === 0) {
        for (const slot of [0,1]) {
          const pose = this.body_states(slot);
          const dx = (slot === 0 ? 2.4 : 3.6)-pose[28], dz = goalZ-pose[30];
          const length = Math.hypot(dx,dz);
          if (length < .12) this.set_input(slot,0,0,false);
          else this.set_input(slot,dx/length,dz/length,false);
        }
      }
      step.call(this);
    };
  });
  await start.click();
  await expect(page.getByLabel('Round phase')).toHaveText('Playing',{timeout:15_000});
  try {
    const beside = await besideVent(page,null);
    await test.info().attach('rooftop-vent',{body:await page.locator('.scene-canvas canvas').screenshot(),contentType:'image/png'});
    await page.evaluate(() => window.dispatchEvent(new Event('retreat-from-vent')));
    const returned = await clearOfVent(page,null);
    await test.info().attach('local-camera',{body:JSON.stringify({beside,returned}),contentType:'application/json'});
  } finally {
    await test.info().attach('last-local',{body:JSON.stringify(await ventCamera(page,null)),contentType:'application/json'});
  }
});
