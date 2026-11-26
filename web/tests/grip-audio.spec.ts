import { expect, test } from '@playwright/test';
import { gripPitches, observeWarningAudio } from './support/warningAudio';

if (process.platform === 'darwin') test.use({launchOptions:{args:['--use-gl=angle','--use-angle=metal']}});

for (const muted of [false,true]) {
  const scenario = muted ? 'muted crate grips retain their visual feedback' : 'local hands sound on actual crate contact and release';
  test(scenario, async ({page}) => {
    await observeWarningAudio(page);
    await page.goto('/play/');
    if (muted) await page.getByRole('button',{name:'Mute game audio'}).click();
    await page.getByRole('button',{name:'Start match'}).click();
    await expect(page.getByLabel('Round phase')).toHaveText('Playing',{timeout:15_000});
    await page.locator('.scene-canvas canvas').focus();
    await page.keyboard.down('KeyJ');
    await page.keyboard.down('KeyK');
    const clock = page.locator('.match-round strong');
    await expect(clock).not.toHaveText(await clock.innerText());
    expect(await gripPitches(page)).toEqual([]);
    await page.keyboard.down('KeyS');
    try {
      await expect(page.getByLabel('Coral grips')).toContainText('hand on crate',{timeout:15_000});
      if (!muted) await expect.poll(() => gripPitches(page)).toContain(320);
    } finally {
      await page.keyboard.up('KeyS');
      await page.keyboard.up('KeyJ');
      await page.keyboard.up('KeyK');
    }
    await expect(page.getByLabel('Coral grips')).toHaveCount(0);
    if (muted) expect(await gripPitches(page)).toEqual([]);
    else await expect.poll(() => gripPitches(page)).toContain(180);
  });
}

test('grip cues render distinct short envelopes with bounded peaks and stereo falloff', async ({page}) => {
  await page.goto('/play/');
  const samples = await page.evaluate(async () => {
    const {playGrip} = await import(/* @vite-ignore */ String('/src/audio/gripSound.ts'));
    const render = async (kind:string, pan:number, distance:number) => {
      const ctx = new OfflineAudioContext(2,12000,48000);
      playGrip(ctx,{kind,pan,distance});
      const pcm = await ctx.startRendering();
      return [0,1].map(i => {
        const buffer = pcm.getChannelData(i);
        return {peak:buffer.reduce((peak,x) => Math.max(peak,Math.abs(x)),0),
          tail:buffer.reduce((last,x,index) => Math.abs(x) > 0.0001 ? index : last,0) / pcm.sampleRate};
      });
    };
    return Promise.all([render('catch',-1,8),render('strain',1,8),render('release',1,8),render('catch',-1,32)]);
  });
  const [caught,strain,release,far] = samples;
  const envelope = [caught![0]!,strain![1]!,release![1]!];
  for (const channel of envelope) {
    expect(channel.peak).toBeGreaterThan(0.02);
    expect(channel.peak).toBeLessThan(0.09);
    expect(channel.tail).toBeLessThan(0.20);
  }
  expect(envelope[0]!.tail).toBeLessThan(envelope[2]!.tail);
  expect(envelope[2]!.tail).toBeLessThan(envelope[1]!.tail);
  expect(caught![1]!.peak).toBeLessThan(0.00001);
  expect(strain![0]!.peak).toBeLessThan(0.00001);
  expect(far![0]!.peak / caught![0]!.peak).toBeCloseTo(0.25,1);
  await test.info().attach('grip-pcm',{body:JSON.stringify(samples),contentType:'application/json'});
});
