import { execFileSync } from 'node:child_process';
import { cpus, platform, release } from 'node:os';
import { fileURLToPath } from 'node:url';
import { chromium } from '@playwright/test';
import { build } from 'vite';
import { monitor } from './monitor.mjs';
import { startServer } from './server.mjs';

const args = process.argv.slice(2);
if (args.length && !(args.length === 1 && args[0] === '--quick')) throw new Error('Usage: npm run benchmark:endurance -- [--quick]');
const seconds = args.length ? 90 : 3600;
const root = fileURLToPath(new URL('../../../',import.meta.url));
execFileSync('cargo',['build','--locked','--release','-p','brawler-server'],{cwd:root,stdio:'inherit'});
const bundle = await build({configFile:false,logLevel:'silent',build:{write:false,
  lib:{entry:fileURLToPath(new URL('client.ts',import.meta.url)),formats:['iife'],name:'EnduranceClient'}}});
const source = bundle[0].output[0].code;
const server = await startServer();
try {
  const browser = await chromium.launch();
  try {
    const catalog = server.identity;
    console.log(JSON.stringify({seconds,cpu:cpus()[0].model,platform:platform(),release:release(),browser:browser.version(),
      serverSha256:server.serverSha256,engine:catalog.engineHash,map:catalog.catalogHash}));
    const pages = [];
    let code = null;
    for (let slot = 0; slot < 8; slot++) {
      const context = await browser.newContext();
      const page = await context.newPage();
      page.on('console', message => { if (message.type() === 'error') console.error('Seat',slot,message.text()); });
      await page.goto(server.origin+'/healthz');
      await page.addScriptTag({content:source});
      code = await page.evaluate(({slot,code}) => window.EnduranceClient.admit(slot,code),{slot,code});
      pages.push(page);
    }
    await Promise.all(pages.map((p,slot) => p.evaluate(({slot,code,hashes}) => {
      window.driver = window.EnduranceClient.play(slot,code,hashes);
    },{slot,code,hashes:catalog.engineHash+catalog.catalogHash})));
    await monitor(pages,server,seconds);
    console.log(JSON.stringify({result:'passed',seconds}));
  } finally { await browser.close(); }
} finally { await server.close(); }
