import { expect, test } from '@playwright/test';

test('a roof footfall pans, attenuates with distance and finishes quietly', async ({page},info) => {
  await page.goto('/play/');
  const measures = await page.evaluate(async () => {
    const {playFootstep} = await import(/* @vite-ignore */ String('/src/audio/footstepSound.ts'));
    const results = [];
    for (const [pan,distance] of [[-1,8],[1,8],[-1,32]]) {
      const context = new OfflineAudioContext(2,6000,48000);
      playFootstep(context,{pan,distance,tick:20});
      const buffer = await context.startRendering();
      const channels = [buffer.getChannelData(0),buffer.getChannelData(1)];
      results.push({
        peaks:channels.map(c => c.reduce((peak,v) => Math.max(peak,Math.abs(v)),0)),
        energy:channels.reduce((total,c) => c.reduce((sum,v) => sum + v*v,total),0),
        tail:channels.every(c => c.slice(4200).every(v => v === 0))
      });
    }
    return results;
  });
  await info.attach('footstep-levels.json',{body:JSON.stringify(measures),contentType:'application/json'});
  const [left,right,far] = measures;
  expect(left!.peaks[0]).toBeGreaterThan(.01);
  expect(left!.peaks[0]).toBeLessThan(.05);
  expect(left!.peaks[1]).toBeLessThan(.00001);
  expect(right!.peaks[1]).toBeCloseTo(left!.peaks[0]!,5);
  expect(right!.peaks[0]).toBeLessThan(.00001);
  expect(far!.energy / left!.energy).toBeCloseTo(1/16,4);
  expect(measures.every(m => m.tail)).toBe(true);
});
