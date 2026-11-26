import { expect, test } from './support/skylineTest';
import { silhouettePixels } from './support/silhouettePixels';

test('Space pulls Coral from the collapsed walkway back into the same live round', async ({ page }) => {
  await page.goto('/play/');
  await expect(page.getByRole('button', { name: 'Start match' })).toBeEnabled();
  await page.evaluate(async () => {
    const { LocalDuel } = await import(/* @vite-ignore */ String('/src/generated/wasm/brawler_wasm.js'));
    const step = LocalDuel.prototype.step;
    const setJump = LocalDuel.prototype.set_jump;
    let ready = false;
    let pulling = false;
    LocalDuel.prototype.set_jump = function (owner: number, held: boolean) {
      if (owner === 0 && held) pulling = true;
      setJump.call(this, owner, held);
    };
    LocalDuel.prototype.step = function () {
      // Reproduce the full approach with ordinary inputs and ticks, then wait at the
      // verified catch for the canvas to receive the player's keyboard event.
      if (!ready) {
        for (let i = 0; i < 180; i++) step.call(this);
        for (let tick = 0; tick < 7320; tick++) {
          if (tick === 6800) this.set_input(0, -1, 0, false);
          if (tick === 7050) this.set_input(0, 1, 0, false);
          if (tick === 7078) this.set_input(0, 0, 0, false);
          if (tick === 7160) { this.set_hand(0, 0, true); this.set_hand(0, 1, true); }
          step.call(this);
        }
        ready = true;
      } else if (pulling) {
        for (let i = 0; i < 4; i++) step.call(this);
      }
      const pelvis = this.body_states(0).slice(28, 31);
      document.documentElement.dataset.testFighter = JSON.stringify([Number(this.tick()), ...pelvis]);
    };
  });
  await page.getByRole('button', { name: 'Start match' }).click();
  await expect(page.getByLabel('Coral grips')).toContainText('roof edge', { timeout: 14000 });
  await expect(page.getByLabel('Coral grips')).toContainText('Hold Space to pull up');
  const canvas = page.locator('canvas');
  const hanging = await canvas.screenshot({path: test.info().outputPath('west-edge-hang.png')});
  const coral = [255, 80, 71], blue = [36, 174, 235];
  const concealed = await silhouettePixels(page, hanging, coral);
  expect(concealed.count).toBeGreaterThan(40);
  expect(concealed.height).toBeGreaterThan(8);
  expect((await silhouettePixels(page, hanging, blue)).count).toBe(0);
  await page.setViewportSize({width: 390, height: 844});
  const portrait = await canvas.screenshot({path: test.info().outputPath('west-edge-portrait.png')});
  expect((await silhouettePixels(page, portrait, coral)).height).toBeGreaterThan(6);
  await page.setViewportSize({width: 1440, height: 900});
  await page.keyboard.press('KeyZ');
  await page.keyboard.press('KeyZ');
  await expect.poll(async () => (await silhouettePixels(page, await canvas.screenshot(), coral)).count)
    .toBeLessThan(concealed.count / 10);
  await page.keyboard.down('Space');
  await expect.poll(() => page.evaluate(() => {
    const [tick, x, y] = JSON.parse(document.documentElement.dataset.testFighter!);
    return tick >= 7720 && x > -10 && y > 0.55;
  }), { timeout: 20000 }).toBe(true);
  await page.keyboard.up('Space');
  await expect(page.getByLabel('Round phase')).toHaveText('Playing');
  await expect(page.getByLabel('Blue score')).toHaveText('0');
  await expect(page.getByLabel('Coral grips')).toHaveCount(0);
});
