import {expect,test} from '@playwright/test';

test('landing impulse controls the bounded stereo thump and decays to silence', async ({page}, info) => {
  await page.goto('/play/');
  const measures = await page.evaluate(async () => {
    const {playLanding} = await import(/* @vite-ignore */ String('/src/audio/landingSound.ts'));
    const results = [];
    for (const [impulse, pan, distance] of [[12,0,8],[90,-1,8],[90,1,8],[255,-1,8],[90,-1,32]]) {
      const context = new OfflineAudioContext(2,12000,48000);
      playLanding(context, {impulse,pan,distance});
      const render = await context.startRendering();
      const samples = [render.getChannelData(0),render.getChannelData(1)];
      results.push({peaks:samples.map(c => c.reduce((peak,v) => Math.max(peak,Math.abs(v)),0)),
        energy:samples.reduce((total,c) => c.reduce((sum,v) => sum+v*v,total),0),
        tail:samples.every(c => c.slice(8100).every(v => v === 0))});
    }
    return results;
  });
  await info.attach('landing-levels.json',{body:JSON.stringify(measures),contentType:'application/json'});
  const [light,left,right,capped,far] = measures;
  expect(left!.peaks[0]).toBeGreaterThan(0.09);
  expect(left!.peaks[0]).toBeLessThan(0.13);
  expect(left!.peaks[1]).toBeLessThan(.00001);
  expect(right!.peaks[1]).toBeCloseTo(left!.peaks[0]!,5);
  expect(right!.peaks[0]).toBeLessThan(.00001);
  expect(light!.energy).toBeLessThan(left!.energy);
  expect(capped).toEqual(left);
  expect(far!.energy / left!.energy).toBeCloseTo(1/16,4);
  expect(measures.every(m => m.tail)).toBe(true);
});
