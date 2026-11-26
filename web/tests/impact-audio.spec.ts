import { expect, test } from './support/skylineTest';
import { impactCount, observeWarningAudio } from './support/warningAudio';

test('damaging contacts produce a short stereo thud with bounded strength and distance falloff', async ({page}) => {
  await page.goto('/play/');
  const results = await page.evaluate(async () => {
    const {playImpact} = await import(/* @vite-ignore */ String('/src/audio/impactSound.ts'));
    const results = [];
    for (const [damage, pan, distance] of [[35,-1,8],[35,1,8],[1,0,8],[35,0,32],[350,-1,8]]) {
      const context = new OfflineAudioContext(2,12000,48000);
      playImpact(context, {damage, pan, distance});
      const samples = await context.startRendering();
      const channels = [samples.getChannelData(0),samples.getChannelData(1)];
      results.push({
        peaks: channels.map(c => c.reduce((peak,v) => Math.max(peak, Math.abs(v)), 0)),
        energy: channels.reduce((total,c) => c.reduce((sum,v) => sum+v*v, total),0),
        silentTail: channels.every(c => c.slice(7200).every(v => v === 0)),
      });
    }
    return results;
  });
  const [left,right,light,far,capped] = results;
  expect(left!.peaks[0]).toBeGreaterThan(0.07);
  expect(left!.peaks[0]).toBeLessThan(0.16);
  expect(left!.peaks[1]).toBeLessThan(0.00001);
  expect(right!.peaks[0]).toBeLessThan(0.00001);
  expect(right!.peaks[1]).toBeCloseTo(left!.peaks[0]!,5);
  expect(capped).toEqual(left);
  expect(light!.energy).toBeLessThan(left!.energy);
  expect(far!.energy / left!.energy).toBeCloseTo(1 / 16,4);
  expect(results.every(r => r.silentTail)).toBe(true);
});

test('an out-of-reach punch and headbutt at the default spawns stay silent', async ({page}) => {
  await observeWarningAudio(page);
  await page.goto('/play/');
  await page.getByRole('button',{name:'Start match'}).click();
  await expect(page.getByLabel('Round phase')).toHaveText('Playing',{timeout:15000});
  const clock = page.locator('.match-round strong');
  await page.locator('.scene-canvas canvas').focus();
  const before = await clock.innerText();
  for (const key of ['KeyJ', 'KeyK', 'KeyQ']) await page.keyboard.press(key);
  await expect(clock).not.toHaveText(before, {timeout:10000});
  expect(await impactCount(page)).toBe(0);
});
