import { expect, test } from '@playwright/test';
import { observeArenaMeshes } from './support/arenaMeshes';
import { delayRelay } from './support/delayRelay';
import { gripMatch } from './support/gripMatch';
import { observeLocalJumpDraws, type JumpDraw } from './support/jumpDraw';
import { measureLink } from './support/measureLink';

if (process.platform === 'darwin') test.use({launchOptions:{args:['--use-gl=angle','--use-angle=metal']}});

test('measure regional keydown to first drawn jump on the local fighter', async ({page,browser,baseURL}, info) => {
  test.slow();
  const relay = await delayRelay(baseURL!);
  relay.profile(60,5);
  const trials: JumpDraw[] = [];
  const roundTripsMs: number[] = [];
  try {
    const readRtt = await measureLink(page,relay.port);
    await observeArenaMeshes(page);
    // A fresh round per sample fixes the character's initial support, health and hazard state.
    for (let trial = 0; trial < 20; trial++) {
      await gripMatch(page,browser,async (_guest,start) => {
        await start();
        await expect.poll(async () => (await readRtt()).length).toBeGreaterThanOrEqual(3);
        await observeLocalJumpDraws(page);
        await page.waitForFunction(() => window.readyForJump,undefined,{timeout:5000});
        await page.keyboard.press('Space');
        await page.waitForFunction(() => {
          const jump = window.jumpDraws[0];
          return jump && (jump.riseMs !== undefined || performance.now() - jump.keyMs >= 1500);
        });
        trials.push(await page.evaluate(() => window.jumpDraws[0]!));
        roundTripsMs.push(...await readRtt());
      });
    }
    expect(trials.every(j => j.sendMs !== undefined && j.riseMs !== undefined && j.sendMs < j.riseMs)).toBe(true);
  } finally {
    await info.attach('regional-jump-feedback',{body:JSON.stringify({roundTripsMs,trials}),contentType:'application/json'});
    await relay.close();
  }
});
