import { expect, test } from '@playwright/test';
import { observeArenaMeshes } from './support/arenaMeshes';
import { delayRelay } from './support/delayRelay';
import { gripMatch } from './support/gripMatch';
import { observeHeadbutt, type HeadbuttTrial } from './support/headbuttDraw';
import { measureLink } from './support/measureLink';

test.use({viewport:{width:1280,height:720}});
if (process.platform === 'darwin') test.use({launchOptions:{args:['--use-gl=angle','--use-angle=metal']}});

test('regional headbutt input moves the drawn head after server acceptance', async ({page,browser,baseURL},info) => {
  test.slow();
  const relay = await delayRelay(baseURL!);
  relay.profile(60,5);
  const measure = process.env.BRAWLER_CHECK_REGIONAL_HEADBUTT === '1';
  const trials: HeadbuttTrial[] = [], roundTripsMs: number[] = [];
  try {
    const rtt = await measureLink(page,relay.port);
    await observeArenaMeshes(page);
    for (let room = 0; room < (measure ? 20 : 1); room++) {
      await gripMatch(page,browser,async (_guest,start) => {
        await start();
        await expect.poll(async () => (await rtt()).length).toBeGreaterThanOrEqual(3);
        await observeHeadbutt(page);
        await page.waitForFunction(() => window.headSettled,undefined,{timeout:5000});
        await page.keyboard.press('KeyQ');
        await page.waitForFunction(() => {
          const trial = window.headbuttTrial;
          return trial && (trial.drawnMs !== undefined && trial.acceptedMs !== undefined || performance.now()-trial.keyMs > 1500);
        });
        const trial = await page.evaluate(() => window.headbuttTrial!);
        trials.push(trial);
        roundTripsMs.push(...await rtt());
        expect(trial.sentMs).toBeDefined();
        expect(trial.acceptedMs).toBeGreaterThan(trial.sentMs!);
        expect(trial.drawnMs).toBeGreaterThanOrEqual(trial.acceptedMs!);
      });
    }
    if (measure) {
      const ordered = trials.map(t => t.drawnMs!).sort((a,b) => a-b);
      expect(ordered[Math.ceil(.95 * ordered.length)-1]).toBeLessThanOrEqual(150);
    }
  } finally {
    await info.attach('regional-headbutt-feedback',{body:JSON.stringify({trials,roundTripsMs}),contentType:'application/json'});
    await relay.close();
  }
});
