import { expect, test, type Page } from '@playwright/test';
import { decodeSnapshot, type Snapshot } from '../src/net/snapshot';
import { gripMatch } from './support/gripMatch';

if (process.platform === 'darwin') test.use({launchOptions: {args: ['--use-gl=angle', '--use-angle=metal']}});

function observeRescue(page: Page) {
  const state = {tick: 0, x: 0, y: 0, heldBelow: false, stood: false};
  page.on('websocket', socket => socket.on('framereceived', ({payload}) => {
    if (!Buffer.isBuffer(payload) || payload[0] !== 2) return;
    const frame = decodeSnapshot(payload);
    const [x,y] = frame.bodies.get(3)!.position;
    const grips = frame.fighters[0]!.grips;
    state.tick = frame.arenaTick;
    state.x = x;
    state.y = y;
    state.heldBelow ||= frame.phase === 1 && frame.number === 1 && grips.includes(65537) && x < -10 && y < -.4;
    state.stood ||= state.heldBelow && frame.phase === 1 && frame.number === 1 && grips.every(id => id === 0)
      && x > -9.8 && y > .6 && frame.fighters.every(f => f.consciousness === 0 && f.score === 0);
  }));
  return state;
}

async function walkOntoWestPanel(host: Page) {
  // Only the approach axes are steered from server positions. The player supplies hand
  // and pull-up inputs through the canvas and the arena runs its ordinary two-minute schedule.
  await host.routeWebSocket(url => url.pathname === '/ws', client => {
    const server = client.connectToServer();
    let latest: Snapshot | undefined;
    let stage: 'out' | 'back' | 'parked' = 'out';
    const pending = new Set<ReturnType<typeof setTimeout>>();
    let delayed = false;
    const forward = (send: () => void) => {
      if (!delayed) { send(); return; }
      // A fixed delay in both directions preserves message order and adds 120 ms
      // to a round trip during the fall, rescue and return to player control.
      const timer = setTimeout(() => { pending.delete(timer); send(); }, 60);
      pending.add(timer);
    };
    const clear = () => { for (const timer of pending) clearTimeout(timer); pending.clear(); };
    client.onClose((code, reason) => { clear(); server.close({code, reason}); });
    server.onClose((code, reason) => { clear(); client.close({code, reason}); });
    server.onMessage(message => {
      if (Buffer.isBuffer(message) && message[0] === 2) {
        latest = decodeSnapshot(message);
        delayed ||= latest.arenaTick >= 7100;
      }
      forward(() => client.send(message));
    });
    client.onMessage(message => {
      if (!latest || latest.arenaTick < 6600 || latest.arenaTick >= 7200 ||
        !Buffer.isBuffer(message) || message[0] !== 1) { forward(() => server.send(message)); return; }
      const x = latest.bodies.get(3)!.position[0];
      if (stage === 'out' && x <= -10.76) stage = 'back';
      if (stage === 'back' && x >= -10.38) stage = 'parked';
      const direction = stage === 'out' ? -1 : stage === 'back' ? Math.min(.8, Math.max(.12, (-10.38-x)*2)) : 0;
      const outgoing = Buffer.from(message);
      outgoing.writeInt8(Math.round(127*direction),10);
      outgoing.writeInt8(0,11);
      forward(() => server.send(outgoing));
    });
  });
}

test('a player catches the roof after the west collapse and pulls up in both views', async ({page: host, browser}) => {
  test.setTimeout(160_000);
  await walkOntoWestPanel(host);
  await gripMatch(host, browser, async (guest, start) => {
    const states = [host,guest].map(observeRescue);
    await start();
    await expect.poll(() => states[0]!.tick, {timeout: 125_000, intervals:[250]}).toBeGreaterThanOrEqual(7120);
    await host.keyboard.down('KeyJ');
    await host.keyboard.down('KeyK');
    await expect.poll(() => states.every(s => s.heldBelow), {timeout: 7000, intervals:[40]}).toBe(true);
    await expect(host.getByText('Hold Space to pull up', {exact: true})).toBeVisible();
    await expect(guest.getByLabel('Mint grips')).toContainText('roof edge');
    await expect(guest.getByText('Hold Space to pull up', {exact: true})).toHaveCount(0);
    await host.keyboard.down('Space');
    await expect.poll(() => states.every(s => s.stood), {timeout: 9000}).toBe(true);
    await host.keyboard.up('Space');
    await host.keyboard.up('KeyJ');
    await host.keyboard.up('KeyK');
    const before = states.map(s => s.x);
    await host.keyboard.down('KeyD');
    try {
      await expect.poll(() => states.every((s,i) => s.x > before[i]! + .8)).toBe(true);
    } finally { await host.keyboard.up('KeyD'); }
    for (const page of [host,guest]) {
      await expect(page.getByLabel('Round phase')).toHaveText('Playing');
      await expect(page.getByLabel('Mint grips')).toHaveCount(0);
      await expect(page.getByLabel('Mint score')).toHaveText('0');
    }
    await expect(host.getByText('Hold Space to pull up', {exact: true})).toHaveCount(0);
  });
});
