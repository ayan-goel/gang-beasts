import { expect, test } from '@playwright/test';

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
