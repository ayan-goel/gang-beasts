import { breakawayPans, observeWarningAudio, warningSamples } from './support/warningAudio';
import { expect, test } from '@playwright/test';

// The local warning follows 5,400 physics ticks on the render clock. Use the Mac GPU for this real-time scene.
if (process.platform === 'darwin') test.use({launchOptions:{args:['--use-gl=angle','--use-angle=metal']}});

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

test('local warnings and breakaway audio follow physical arena time and survive pause', async ({page}) => {
  test.setTimeout(240_000);
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
  expect(await breakawayPans(page)).toEqual([]);
  await expect(page.getByText('East walkway failing')).toBeVisible({timeout:65_000});
  await expect.poll(() => breakawayPans(page)).toHaveLength(1);
  await expect(page.getByText('High winds approaching')).toBeVisible({timeout:65_000});
  await expect.poll(() => breakawayPans(page)).toHaveLength(2);
  const [west,east] = await breakawayPans(page);
  expect(west! * east!).toBeLessThan(0);
});

test('match warnings, condition and round cues render bounded pulse patterns in the browser audio graph', async ({page}) => {
  await page.goto('/play/');
  const shapes = await page.evaluate(async () => {
    const {playMatchTone} = await import(/* @vite-ignore */ String('/src/audio/matchTone.ts'));
    const patterns = [];
    for (const stage of ['west_warning','east_warning','gust_warning','gust','knocked_out','recovering','count','go','round_win','draw','match_win']) {
      const context = new OfflineAudioContext(1, 48000, 48000);
      playMatchTone(context, stage);
      const samples = (await context.startRendering()).getChannelData(0);
      let pulses = 0, previous = false, peak = 0;
      for (let i = 0; i < samples.length; i += 480) {
        let energy = 0;
        for (const value of samples.subarray(i,i+480)) { energy += value*value; peak = Math.max(peak, Math.abs(value)); }
        const audible = energy / 480 > 0.00001;
        if (audible && !previous) pulses++;
        previous = audible;
      }
      patterns.push({stage, pulses, peak, silentEnd:samples.slice(-4800).every(x => x === 0)});
    }
    return patterns;
  });
  expect(shapes.map(s => s.pulses)).toEqual([2,3,2,1,1,2,1,1,2,2,3]);
  for (const s of shapes) {
    expect(s.peak).toBeGreaterThan(0.05);
    const cap = ['count','go','round_win','draw','match_win'].includes(s.stage) ? 0.07
      : s.stage === 'knocked_out' || s.stage === 'recovering' ? 0.08 : 0.12;
    expect(s.peak).toBeLessThanOrEqual(cap + 0.001);
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
