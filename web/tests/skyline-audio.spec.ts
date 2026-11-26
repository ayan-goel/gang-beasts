import { observeWarningAudio, warningSamples } from './support/warningAudio';
import { expect, test } from '@playwright/test';

test('muting Skyline persists on reload and into private rooms', async ({page}) => {
  await page.goto('/play/');
  const mute = page.getByRole('button', {name:'Mute game audio'});
  await expect(mute).toHaveAttribute('aria-pressed','false');
  await mute.focus();
  await page.keyboard.press('Space');
  await expect(mute).toHaveAttribute('aria-pressed','true');
  await page.reload();
  await expect(mute).toHaveAttribute('aria-pressed','true');
  await page.getByRole('link', {name:'Private rooms'}).click();
  await expect(mute).toHaveAttribute('aria-pressed','true');
  await mute.click();
  await expect(mute).toHaveAttribute('aria-pressed','false');
});

test('a local match unlocks audio on start and plays one walkway cue from physical arena time', async ({page}) => {
  test.setTimeout(180_000);
  await observeWarningAudio(page);
  await page.goto('/play/');
  const start = page.getByRole('button', {name:'Start match'});
  await expect(start).toBeEnabled();
  expect(await page.evaluate(() => window.skylineAudioProbe.contexts)).toBe(0);
  await start.click();
  await expect(page.getByText('West walkway failing')).toBeVisible({timeout:155_000});
  await expect.poll(() => warningSamples(page)).toEqual([{wave:'triangle',state:'running'}]);
  await page.getByRole('button',{name:'Pause',exact:true}).click();
  await page.getByRole('button',{name:'Resume match'}).click();
  await expect(page.getByText('West walkway failing')).toBeVisible();
  expect(await warningSamples(page)).toHaveLength(1);
});

test('the four original warnings render distinct, bounded pulse patterns in the browser audio graph', async ({page}) => {
  await page.goto('/play/');
  const shapes = await page.evaluate(async () => {
    const {playWarningTone} = await import(/* @vite-ignore */ String('/src/audio/warningTone.ts'));
    const patterns = [];
    for (const stage of ['west_warning','east_warning','gust_warning','gust']) {
      const context = new OfflineAudioContext(1, 48000, 48000);
      playWarningTone(context, stage);
      const samples = (await context.startRendering()).getChannelData(0);
      let pulses = 0, previous = false, peak = 0;
      for (let i = 0; i < samples.length; i += 480) {
        let energy = 0;
        for (const value of samples.subarray(i,i+480)) { energy += value*value; peak = Math.max(peak, Math.abs(value)); }
        const audible = energy / 480 > 0.00001;
        if (audible && !previous) pulses++;
        previous = audible;
      }
      patterns.push({pulses, peak, silentEnd:samples.slice(-4800).every(x => x === 0)});
    }
    return patterns;
  });
  expect(shapes.map(s => s.pulses)).toEqual([2,3,2,1]);
  for (const s of shapes) {
    expect(s.peak).toBeGreaterThan(0.07);
    expect(s.peak).toBeLessThanOrEqual(0.13);
    expect(s.silentEnd).toBe(true);
  }
});

test('a player can start and mute a match when sound and persistent storage are unavailable', async ({page}) => {
  const errors: string[] = [];
  page.on('pageerror', error => errors.push(error.message));
  await page.addInitScript(() => {
    Object.defineProperty(window, 'AudioContext', {value:undefined});
    Object.defineProperty(window, 'localStorage', {get() { throw new DOMException('Disabled','SecurityError'); }});
  });
  await page.goto('/play/');
  const mute = page.getByRole('button',{name:'Mute game audio'});
  await mute.click();
  await expect(mute).toHaveAttribute('aria-pressed','true');
  await page.getByRole('button',{name:'Start match'}).click();
  await expect(page.getByLabel('Round phase')).toHaveText('Playing',{timeout:15_000});
  expect(errors).toEqual([]);
});
