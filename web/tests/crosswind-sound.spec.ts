import { expect, test } from '@playwright/test';

test('the gust loops through effects with a continuous seam, independent music gain and a bounded level', async ({page}) => {
  await page.goto('/play/');
  for (const rate of [44100, 48000]) {
    const result = await page.evaluate(async rate => {
      const {MatchMix} = await import(/* @vite-ignore */ String('/src/audio/MatchMix.ts'));
      const render = async (master: number, effects: number, music: number, muted = false) => {
        const context = new OfflineAudioContext(1, rate * 5, rate);
        const mix = new MatchMix(context, {master, effects, music, muted});
        mix.setWind(true);
        mix.setWind(true);
        const samples = (await context.startRendering()).getChannelData(0);
        mix.close();
        return samples;
      };
      const full = await render(100,100,0);
      const low = await render(50,30,100);
      const muted = await render(100,100,0,true);
      const off = await render(100,0,100);
      const peak = (values: Float32Array) => values.reduce((p,s) => Math.max(p,Math.abs(s)),0);
      const rms = (values: Float32Array) => Math.sqrt(values.reduce((sum,x) => sum+x*x,0) / values.length);
      const seam = 4 * rate;
      return {peak:peak(full), quietRatio:peak(low)/peak(full), muted:peak(muted), off:peak(off),
        firstSecond:rms(full.subarray(0,rate)), fifthSecond:rms(full.subarray(seam)),
        seamRms:rms(full.subarray(seam-200,seam+200)), seamStep:Math.abs(full[seam]! - full[seam-1]!)};
    },rate);
    expect(result.peak).toBeGreaterThan(0.015);
    expect(result.peak).toBeLessThan(0.075);
    expect(Math.min(result.firstSecond, result.fifthSecond, result.seamRms)).toBeGreaterThan(.002);
    expect(result.seamStep).toBeLessThan(.01);
    expect(result.quietRatio).toBeCloseTo(.15,5);
    expect([result.off,result.muted]).toEqual([0,0]);
    await test.info().attach('crosswind-'+rate,{body:JSON.stringify(result),contentType:'application/json'});
  }
});
