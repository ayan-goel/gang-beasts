import assert from 'node:assert/strict';
import test from 'node:test';
import { memoryPlateau } from './memory.mjs';

const series = f => Array.from({length:61},(_,minute) => ({elapsedS:minute*60,rssKiB:f(minute)}));

test('warm-up growth and an isolated later spike preserve the steady-state median', () => {
  const samples = series(m => m < 30 ? 5000+m*100 : m === 52 ? 12100 : 8000+(m%3)*16);
  const report = memoryPlateau(samples);
  assert.equal(report.plateau,true);
  assert.equal(report.peakKiB,12100);
  assert.deepEqual(report.lateMediansKiB,[8016,8016,8016]);
});

test('sustained growth and missing observation windows cannot pass as a plateau', () => {
  const samples = series(m => 8000+80*m);
  assert.equal(memoryPlateau(samples).plateau,false);
  assert.throws(() => memoryPlateau(samples.filter((_,i) => i < 43 || i > 49)),/samples/);
  assert.throws(() => memoryPlateau(samples.slice(0,58)),/hour/);
});
