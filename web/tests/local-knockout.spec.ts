import { expect, test } from '@playwright/test';
import { observeWarningAudio, warningSamples } from './support/warningAudio';

test('shared-keyboard combat shows the downed fighter and sounds its two condition changes', async ({page}) => {
  await observeWarningAudio(page);
  await page.goto('/play/');
  const start = page.getByRole('button',{name:'Start match'});
  await expect(start).toBeEnabled();
  await page.evaluate(async () => {
    const {LocalDuel} = await import(/* @vite-ignore */ String('/src/generated/wasm/brawler_wasm.js'));
    const readPose = LocalDuel.prototype.body_states;
    const canvas = document.querySelector('canvas')!;
    const held = new Set<string>();
    let finished = false, nextSwing = 0, nextHeadbutt = 0, left = true;
    const key = (code: string, down: boolean) => canvas.dispatchEvent(new KeyboardEvent(
      down ? 'keydown' : 'keyup', {code, bubbles:true, cancelable:true}));
    const tap = (code: string) => { key(code,true); key(code,false); };
    // Read pelvis positions to steer two keyboard players through the existing canvas controls.
    LocalDuel.prototype.body_states = function (slot: number) {
      const pose = readPose.call(this,slot);
      if (slot !== 1 || finished || document.querySelector('[aria-label="Round phase"]')?.textContent !== 'Playing') return pose;
      const target = readPose.call(this,0);
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
      return pose;
    };
  });
  await start.click();
  await expect(page.getByText('Blue knocked out',{exact:true})).toBeVisible({timeout:90_000});
  const recovery = page.getByText('Blue getting up',{exact:true});
  await expect.poll(() => recovery.isVisible(),{intervals:[50],timeout:15_000}).toBe(true);
  await expect(recovery).toHaveCount(0);
  expect(await warningSamples(page)).toEqual([{wave:'triangle',state:'running'}, {wave:'sine',state:'running'}]);
});
