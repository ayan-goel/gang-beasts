import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { createHash } from 'node:crypto';
import { fileURLToPath } from 'node:url';
import { chromium } from '@playwright/test';
import { preview } from 'vite';
import { splitCapture, summarize, unpack } from './frameSamples.mjs';
import { installObserver } from './renderObserver.mjs';
import { replayRoom } from './replayRoom.mjs';

const root = fileURLToPath(new URL('../../', import.meta.url));
const web = fileURLToPath(new URL('../', import.meta.url));
if (process.argv.slice(2).some(argument => argument !== '--quick')) {
  throw new Error('Usage: node benchmarks/renderProfile.mjs [--quick]');
}
const quick = process.argv.includes('--quick');
const seconds = quick ? 15 : 120, warmup = quick ? 5 : 30, repetitions = quick ? 1 : 5;
const presets = ['medium','high','low'];
const {identity,bytes:trace} = splitCapture(execFileSync('cargo',
  ['run','--quiet','--release','--locked','-p','brawler-server','--example','render_trace'],
  {cwd:root, maxBuffer:32 * 1024 * 1024, stdio:['ignore','pipe','inherit']}));
const packets = unpack(trace);
assert.equal(packets.length, 151 * 30);
const traceHash = createHash('sha256').update(trace).digest('hex');
const site = await preview({root:web, preview:{host:'127.0.0.1', port:0, open:false}});
try {
  const browser = await chromium.launch({args: process.platform === 'darwin' ? ['--use-gl=angle','--use-angle=metal'] : []});
  try {
    const viewport = {width:1440,height:900}, dpr = 2;
    for (let repeat = 0; repeat < repetitions; repeat++) {
      for (let index = 0; index < presets.length; index++) {
        const preset = presets[(repeat + index) % presets.length];
        const context = await browser.newContext({viewport,deviceScaleFactor:dpr});
        try {
          const page = await context.newPage();
          await page.addInitScript(installObserver, {preset,warmup,seconds});
          await page.bringToFront();
          const replay = await replayRoom(page, 'http://127.0.0.1:' + site.httpServer.address().port, packets, identity, warmup, seconds);
          const data = await page.evaluate(() => window.skylineProfile);
          assert.ok(!data.hidden && data.intervals.length >= seconds, 'Visible match must render throughout the sample');
          assert.equal(data.shadow, {low:512,medium:1024,high:2048}[preset]);
          assert.equal(data.skeletons, 8);
          assert.equal(data.callbackTimes.length, data.drawTimes.length);
          const elapsed = data.intervals.reduce((total,dt) => total+dt, 0);
          assert.ok(elapsed >= (seconds - 0.5) * 1000, 'Incomplete sampling window');
          console.log(JSON.stringify({preset,run:repeat+1, browser:browser.version(), gpu:data.gpu, viewport,dpr,
            buffer:data.buffer,shadow:data.shadow,warmup,seconds,traceHash,...replay,
            intervalMs:summarize(data.intervals),submissionIntervalMs:summarize(data.submissionIntervals),
            callbackCpuMs:summarize(data.callbackTimes),renderCpuMs:summarize(data.drawTimes),
            calls:summarize(data.calls),triangles:summarize(data.triangles),
            overBudget:data.intervals.filter(dt => dt > 1000 / 60).length,
            over25ms:data.intervals.filter(dt => dt > 25).length}));
        } finally { await context.close(); }
      }
    }
  } finally { await browser.close(); }
} finally { await site.close(); }
