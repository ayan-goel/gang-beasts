import { setTimeout as delay } from 'node:timers/promises';
import { memoryPlateau } from './memory.mjs';

export async function monitor(pages,server,seconds) {
  const read = () => Promise.all(pages.map(p => p.evaluate(() => window.driver.sample())));
  const validate = samples => {
    for (const [slot,s] of samples.entries()) {
      if (s.error || s.silentMs > 12000) throw new Error('Seat '+slot+': '+(s.error || 'snapshot stream stalled'));
    }
  };
  const startup = performance.now();
  for (;;) {
    const samples = await read();
    validate(samples);
    if (samples.every(s => s.applied > 0)) break;
    if (performance.now() - startup > 25000) throw new Error('Eight input acknowledgements never arrived');
    await delay(100);
  }
  const start = performance.now();
  const memory = [];
  let reportAt = 0, interrupted = false, previous = Array(8).fill(0);
  for (;;) {
    const elapsed = (performance.now()-start)/1000;
    const samples = await read();
    validate(samples);
    if (!interrupted && elapsed >= 5) interrupted = await pages[7].evaluate(() => window.driver.interrupt());
    if (elapsed >= reportAt || elapsed >= seconds) {
      if (reportAt && samples.some((s,i) => s.frames <= previous[i])) throw new Error('A seat stopped receiving frames');
      previous = samples.map(s => s.frames);
      const host = samples[0];
      const usage = {elapsedS:Math.round(elapsed*1000)/1000,rssKiB:server.rssKiB()};
      memory.push(usage);
      console.log(JSON.stringify({...usage,
        frames:samples.map(s => s.frames),applied:samples.map(s => s.applied),inputs:samples.map(s => s.inputs),
        rounds:host.rounds,matches:host.matches,phase:host.phase,arenaTick:host.arenaTick,scored:host.scored,gripFrames:host.gripFrames,damageFrames:host.damageFrames,
        gustFrames:host.gustFrames,bytes:host.bytes,maxGapMs:Math.round(Math.max(...samples.map(s => s.maxGapMs))),resumes:samples[7].resumes}));
      reportAt += 60;
    }
    if (elapsed >= seconds) {
      if (!interrupted || samples[7].resumes !== 1 || samples.some(s => s.applied < 20 || s.rounds < 2 || !s.scored || !s.gripFrames || !s.damageFrames)) {
        throw new Error('Missing active combat, round transition or recovery');
      }
      if (samples.some(s => s.frames < seconds * 30 * .95 || s.maxGapMs > 12000)) throw new Error('Incomplete live snapshot coverage');
      if (seconds >= 3600) {
        if (samples.some(s => s.matches < 2 || !s.gustFrames)) throw new Error('No rematch or late hazard');
        const measured = memoryPlateau(memory);
        console.log(JSON.stringify(measured));
        if (!measured.plateau) throw new Error('Server RSS did not plateau');
      }
      return;
    }
    await delay(Math.min(2000,Math.max(1,1000*(seconds-elapsed))));
  }
}
