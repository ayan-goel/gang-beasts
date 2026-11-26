import { expect, test, type Page } from '@playwright/test';
import { buttons, encodeInput, quantizeDirection } from '../src/net/inputFrame';
import { decodeSnapshot, type Snapshot } from '../src/net/snapshot';
import { releaseRoom } from './support/releaseRoom';
import { impactCount, observeWarningAudio, warningSamples } from './support/warningAudio';

async function engage(page: Page, slot: 0 | 1) {
  const phases: number[] = [];
  const transitions: {ticks: number; furthestLimb: number}[] = [];
  const result = {phases, transitions, resumedAction:false};
  await page.routeWebSocket(url => url.pathname === '/ws', browser => {
    const server = browser.connectToServer();
    let sequence = 0, stopped = false, testedWake = false;
    let previous: Snapshot | undefined;
    const last = [-40,-28,-42];
    browser.onMessage(packet => {
      if (!Buffer.isBuffer(packet) || packet[0] !== 1) server.send(packet);
    });
    server.onMessage(packet => {
      // Only supply player intentions. Both browsers receive the original server frames.
      browser.send(packet);
      if (!Buffer.isBuffer(packet) || packet[0] !== 2) return;
      const frame = decodeSnapshot(packet);
      if (frame.phase !== 1) return;
      const condition = frame.fighters[1]!.consciousness;
      if (phases.at(-1) !== condition) {
        phases.push(condition);
        if (previous) {
          let furthestLimb = 0;
          for (let id = 17; id < 32; id++) {
            const before = previous.bodies.get(id)!.position, after = frame.bodies.get(id)!.position;
            furthestLimb = Math.max(furthestLimb,Math.hypot(after[0]-before[0],after[1]-before[1],after[2]-before[2]));
          }
          transitions.push({ticks:frame.tick-previous.tick,furthestLimb});
        }
      }
      previous = frame;
      if (stopped && condition === 0 && frame.fighters[1]!.stamina < 99) result.resumedAction = true;
      if (condition !== 0) stopped = true;
      const own = frame.bodies.get(3 + slot * 16)!.position;
      const other = frame.bodies.get(3 + (1-slot) * 16)!.position;
      const x = other[0]-own[0], z = other[2]-own[2], distance = Math.max(0.001, Math.hypot(x,z));
      const axes = stopped ? [0,0] as const : quantizeDirection(x/distance,z/distance);
      const held = slot === 0 && distance > 1.2 && !stopped ? buttons.run : 0;
      let taps = 0;
      if (!stopped && !slot && distance < 1.1) {
        for (const [i,button] of [buttons.leftHand,buttons.rightHand,buttons.headbutt].entries()) {
          if (frame.tick-last[i]! >= (i===2 ? 42 : 39)) {
            taps |= button;
            last[i] = frame.tick;
          }
        }
      }
      if (slot === 1 && stopped && condition === 0 && !testedWake) {
        taps = buttons.headbutt;
        testedWake = true;
      }
      server.send(Buffer.from(encodeInput({sequence:sequence++, tick:frame.tick, axes, held, pressed:taps, released:taps})));
    });
  });
  return result;
}

test('both players see and hear a server-owned knockout and recovery caused by rooftop combat', async ({page:host,browser}) => {
  const profile = await browser.newContext();
  let invite = '';
  try {
    const guest = await profile.newPage();
    const pages = [host,guest];
    await Promise.all(pages.map(observeWarningAudio));
    const traces = await Promise.all([engage(host,0), engage(guest,1)]);
    await host.goto('/rooms/');
    await host.getByLabel('Player name').fill('Mint');
    await host.getByRole('button',{name:'Create room',exact:true}).click();
    invite = await host.getByLabel('Invite link').inputValue();
    await guest.goto(invite);
    await guest.getByLabel('Player name').fill('Blue');
    await guest.getByRole('button',{name:'Join room',exact:true}).click();
    for (const p of [guest,host]) await p.getByRole('button',{name:'Ready up'}).click();
    await host.getByRole('button',{name:'Start match'}).click();
    await expect.poll(() => traces.every(trace => trace.phases.includes(1)),{timeout:85_000}).toBe(true);
    await Promise.all(pages.map(async p => {
      await expect(p.getByText('Blue knocked out',{exact:true})).toBeVisible();
      await expect.poll(() => p.getByText('Blue getting up',{exact:true}).isVisible(), {intervals:[50],timeout:6000}).toBe(true);
      await expect(p.getByText('Blue getting up',{exact:true})).toHaveCount(0);
      expect(await impactCount(p)).toBeGreaterThan(0);
      await expect.poll(() => warningSamples(p)).toEqual([{wave:'triangle',state:'running'}, {wave:'sine',state:'running'}]);
    }));
    await expect.poll(() => traces.every(trace => trace.resumedAction)).toBe(true);
    for (const trace of traces) {
      expect(trace.phases).toEqual([0,1,2,0]);
      expect(trace.transitions).toHaveLength(3);
      for (const transition of trace.transitions) {
        expect(transition.ticks).toBeLessThanOrEqual(6);
        expect(transition.furthestLimb).toBeLessThan(0.5);
      }
    }
    for (const p of pages) {
      await expect(p.getByLabel('Round number')).toHaveText('Round 1');
      await expect(p.getByLabel('Blue score')).toHaveText('0');
      await expect(p.getByLabel('Mint score')).toHaveText('0');
    }
  } finally {
    try { await releaseRoom(invite,[host.context(),profile]); } finally { await profile.close(); }
  }
});
