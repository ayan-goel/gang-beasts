import { expect, test } from '@playwright/test';
import { observeArenaMeshes } from './support/arenaMeshes';
import { delayRelay } from './support/delayRelay';
import { gripMatch } from './support/gripMatch';
import { measureLink } from './support/measureLink';
import { observePunchDraws, type PunchTrial } from './support/punchDraw';

test.use({viewport:{width:1280,height:720}});
if (process.platform === 'darwin') test.use({launchOptions:{args:['--use-gl=angle','--use-angle=metal']}});

test('regional punch input reaches the server and moves the corresponding drawn hand', async ({page,browser,baseURL}, info) => {
  test.slow();
  const relay = await delayRelay(baseURL!);
  relay.profile(60,5);
  const trials: PunchTrial[] = [], roundTripsMs: number[] = [];
  try {
    const readRtt = await measureLink(page,relay.port);
    await observeArenaMeshes(page);
    for (let room = 0; room < 20; room++) {
      await gripMatch(page,browser,async (_guest,start) => {
        await start();
        await expect.poll(async () => (await readRtt()).length).toBeGreaterThanOrEqual(3);
        await observePunchDraws(page);
        await page.locator('canvas').hover();
        // Counterbalance hand order and input device across independent rooms.
        for (const hand of [room % 2, 1 - room % 2]) {
          await page.waitForFunction(() => window.punchReady,undefined,{timeout:5000});
          if ((Math.floor(room/2) + hand) % 2) await page.keyboard.press(hand ? 'KeyK' : 'KeyJ');
          else {
            const button = hand ? 'right' : 'left';
            await page.mouse.down({button}); await page.mouse.up({button});
          }
          await page.waitForFunction(() => {
            const trial = window.punchTrials.at(-1);
            return trial && (trial.drawMs !== undefined && trial.acceptedMs !== undefined || performance.now() - trial.inputMs > 1500);
          });
          trials.push(await page.evaluate(() => window.punchTrials.at(-1)!));
        }
        roundTripsMs.push(...await readRtt());
      });
    }
    for (const hand of [0,1]) {
      const perHand = trials.filter(t => t.hand === hand);
      expect(perHand).toHaveLength(20);
      expect(perHand.filter(t => t.source === 'mouse')).toHaveLength(10);
      expect(perHand.every(t => t.sendMs !== undefined && t.acceptedMs !== undefined && t.drawMs !== undefined &&
        t.sendMs < t.acceptedMs && t.acceptedMs <= t.drawMs)).toBe(true);
      if (process.env.BRAWLER_CHECK_REGIONAL_PUNCH === '1') {
        const ordered = perHand.map(t => t.drawMs!).sort((a,b) => a-b);
        expect.soft(ordered[Math.ceil(.95 * ordered.length)-1], (hand ? 'Right' : 'Left') + ' hand p95 in ms').toBeLessThanOrEqual(150);
      }
    }
  } finally {
    await info.attach('regional-punch-feedback',{body:JSON.stringify({trials,roundTripsMs}),contentType:'application/json'});
    await relay.close();
  }
});
