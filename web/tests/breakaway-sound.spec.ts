import { expect, test } from '@playwright/test';

test('the panel break renders a bounded stereo crack and decaying ring at browser sample rates', async ({page}) => {
  await page.goto('/play/');
  const captures = await page.evaluate(async () => {
    const {playBreakaway} = await import(/* @vite-ignore */ String('/src/audio/breakawaySound.ts'));
    const readings = [];
    for (const rate of [44100,48000]) {
      for (const [pan,distance] of [[-1,8],[1,24]] as const) {
        const context = new OfflineAudioContext(2,Math.ceil(.5*rate),rate);
        playBreakaway(context,{pan,distance});
        const buffer = await context.startRendering();
        const channel = buffer.getChannelData(pan < 0 ? 0 : 1);
        const amplitude = (from: number, to: number) => {
          let peak = 0;
          for (const sample of channel.subarray(Math.floor(from*rate),Math.floor(to*rate))) peak = Math.max(peak,Math.abs(sample));
          return peak;
        };
        readings.push({rate, pan, peak:amplitude(0,.32), ring:amplitude(.05,.10),
          decay:amplitude(.20,.25), tail:amplitude(.321,.5),
          leakage:buffer.getChannelData(pan < 0 ? 1 : 0).some(sample => Math.abs(sample) > 1e-7)});
      }
    }
    return readings;
  });
  for (const [i,c] of captures.entries()) {
    expect(c.leakage).toBe(false);
    expect(c.tail).toBe(0);
    expect(c.peak).toBeLessThanOrEqual(.11);
    expect(c.ring).toBeGreaterThan(.001);
    expect(c.decay).toBeLessThan(c.ring*.2);
    if (c.pan < 0) expect(c.peak).toBeGreaterThan(.035);
    else expect(c.peak / captures[i-1]!.peak).toBeCloseTo(1/3,5);
  }
  console.log('breakaway_pcm',JSON.stringify(captures));
});
