import { expect, test } from '@playwright/test';
import type { LocalDuel } from '../src/generated/wasm/brawler_wasm';
import { observeWarningAudio, roundPitches } from './support/warningAudio';

test('two same-tick Skyline falls display a draw and carry zero points into round two', async ({page}) => {
  await observeWarningAudio(page);
  await page.goto('/play/');
  const start = page.getByRole('button', {name:'Start match'});
  await expect(start).toBeEnabled();
  await page.evaluate(async () => {
    const glue = performance.getEntriesByType('resource').find(entry =>
      /\/brawler_wasm(?:-[\w-]+)?\.js$/.test(new URL(entry.name).pathname));
    if (!glue) throw new Error('Could not locate the loaded Skyline WASM module');
    const wasm = await import(/* @vite-ignore */ glue.name);
    const step = wasm.LocalDuel.prototype.step;
    // Schedule the same two world-space run commands as the native room so render timing cannot separate the falls.
    wasm.LocalDuel.prototype.step = function(this: LocalDuel) {
      if (this.tick() === 38n) this.set_input(1, 1, 0, true);
      if (this.tick() === 221n) this.set_input(0, -1, 0, true);
      step.call(this);
    };
  });
  await start.click();
  await expect(page.getByText('Round drawn. No point awarded.', {exact:true})).toBeVisible({timeout:35_000});
  await expect.poll(() => roundPitches(page)).toEqual([440,440,660,294]);
  for (const phase of ['Round results', 'Countdown']) {
    await expect(page.getByLabel('Round phase')).toHaveText(phase, {timeout:10_000});
    await expect(page.getByLabel('Coral score')).toHaveText('0');
    await expect(page.getByLabel('Blue score')).toHaveText('0');
  }
  await expect(page.getByLabel('Round number')).toHaveText('Round 2');
});
