import { expect, test } from '@playwright/test';
import { impactCount, observeWarningAudio, warningSamples } from './support/warningAudio';

for (const muted of [false, true]) test('shared-keyboard combat shows the knockout with game audio ' + (muted ? 'muted' : 'enabled'), async ({page}) => {
  await observeWarningAudio(page);
  await page.goto('/play/');
  const start = page.getByRole('button',{name:'Start match'});
  await expect(start).toBeEnabled();
  await page.evaluate(async () => {
    const {LocalDuel} = await import(/* @vite-ignore */ String('/src/generated/wasm/brawler_wasm.js'));
    const step = LocalDuel.prototype.step;
    const canvas = document.querySelector('canvas')!;
    const held = new Set<string>();
    let finished = false, nextSwing = 0, nextHeadbutt = 0, left = true;
    const key = (code: string, down: boolean) => canvas.dispatchEvent(new KeyboardEvent(
      down ? 'keydown' : 'keyup', {code, bubbles:true, cancelable:true}));
    const tap = (code: string) => { key(code,true); key(code,false); };
    // Steer before a fixed simulation tick so render cadence cannot change the scripted encounter.
    LocalDuel.prototype.step = function () {
      // The first Playing tick may share a render batch with the countdown. Begin from
      // a settled stance after the controls have entered the playing phase on every client.
      if (finished || this.tick() < 24n || JSON.parse(this.match_json()).phase !== 'playing') return step.call(this);
      const pose = this.body_states(1);
      const target = this.body_states(0);
      const dx = pose[28]-target[28], dz = pose[30]-target[30];
      const angle = Math.round(Math.atan2(0.18*dx+dz, dx-0.18*dz)/(Math.PI/4))*Math.PI/4;
      const x = Math.round(Math.cos(angle)), z = Math.round(Math.sin(angle));
      const distance = Math.hypot(dx,dz);
      const desired = new Set<string>();
      finished = JSON.parse(this.condition_json(1)).phase !== 'active';
      if (!finished) {
        if (x) for (const k of x > 0 ? ['KeyD','ArrowLeft'] : ['KeyA','ArrowRight']) desired.add(k);
        if (z) for (const k of z > 0 ? ['KeyS','ArrowUp'] : ['KeyW','ArrowDown']) desired.add(k);
        if (distance > 1.2) desired.add('ShiftLeft');
      }
      for (const k of held) if (!desired.has(k)) { key(k,false); held.delete(k); }
      for (const k of desired) if (!held.has(k)) { key(k,true); held.add(k); }
      if (!finished && distance < 1.1) {
        const tick = Number(this.tick());
        if (tick >= nextSwing) { tap(left ? 'KeyJ' : 'KeyK'); left = !left; nextSwing = tick+15; }
        if (tick >= nextHeadbutt) { tap('KeyQ'); nextHeadbutt = tick+42; }
      }
      return step.call(this);
    };
  });
  if (muted) await page.getByRole('button',{name:'Mute game audio'}).click();
  expect(await impactCount(page)).toBe(0);
  await start.click();
  await expect(page.getByText('Blue knocked out',{exact:true})).toBeVisible({timeout:90_000});
  const recovery = page.getByText('Blue getting up',{exact:true});
  await expect.poll(() => recovery.isVisible(),{intervals:[50],timeout:15_000}).toBe(true);
  await expect(recovery).toHaveCount(0);
  expect((await impactCount(page)) > 0).toBe(!muted);
  expect(await warningSamples(page)).toEqual(muted ? [] : [{wave:'triangle',state:'running'}, {wave:'sine',state:'running'}]);
});
