import { expect, test } from '@playwright/test';

test('the rendered master and channel gains independently scale arena music and effects', async ({page}) => {
  await page.goto('/play/');
  const peaks = await page.evaluate(async () => {
    const { MatchMix } = await import(/* @vite-ignore */ String('/src/audio/MatchMix.ts'));
    const { playMatchTone } = await import(/* @vite-ignore */ String('/src/audio/matchTone.ts'));
    const render = async (source: 'music' | 'warning', master: number, music: number, effects: number, muted = false) => {
      const context = new OfflineAudioContext(1, 48000, 48000);
      const mix = new MatchMix(context, {master, music, effects, muted});
      if (source === 'music') mix.setMusic(true);
      else playMatchTone(context, 'west_warning', mix.effects);
      const buffer = await context.startRendering();
      const peak = buffer.getChannelData(0).reduce((p, s) => Math.max(p, Math.abs(s)), 0);
      mix.close();
      return peak;
    };
    return {
      music: await render('music',100,100,0), quietMusic: await render('music',50,40,0),
      warning: await render('warning',100,0,100), quietWarning: await render('warning',50,0,30),
      noMusic: await render('music',100,0,100), noWarnings: await render('warning',100,100,0),
      muted: await render('music',100,100,100,true), zeroMaster: await render('warning',0,100,100),
    };
  });
  expect(peaks.music).toBeGreaterThan(0.025);
  expect(peaks.warning).toBeGreaterThan(0.05);
  expect(peaks.quietMusic / peaks.music).toBeCloseTo(0.20, 5);
  expect(peaks.quietWarning / peaks.warning).toBeCloseTo(0.15, 5);
  expect([peaks.noMusic, peaks.noWarnings, peaks.muted, peaks.zeroMaster]).toEqual([0,0,0,0]);
});
