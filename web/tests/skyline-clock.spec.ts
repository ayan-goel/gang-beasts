import { expect, test } from './support/skylineTest';

test('a slow visible renderer preserves local countdown and results durations', async ({page}) => {
  await page.addInitScript(() => {
    window.requestAnimationFrame = callback => window.setTimeout(() => callback(performance.now()), 250);
    window.cancelAnimationFrame = window.clearTimeout.bind(window);
  });
  await page.goto('/play/');
  await expect(page.getByRole('button', {name:'Start match'})).toBeEnabled();
  await page.getByRole('button', {name:'Start match'}).click();
  await page.evaluate(async () => {
    Object.defineProperty(document, 'hidden', {configurable:true, value:true});
    document.dispatchEvent(new Event('visibilitychange'));
    // Deliver slow frame callbacks while hidden for longer than the entire countdown.
    await new Promise(resolve => setTimeout(resolve, 3500));
  });
  await expect(page.getByLabel('Round phase')).toHaveText('Countdown');
  await page.evaluate(() => {
    Reflect.deleteProperty(document, 'hidden');
    document.dispatchEvent(new Event('visibilitychange'));
  });
  await expect(page.getByLabel('Round phase')).toHaveText('Playing', {timeout:6500});
  await page.keyboard.down('ShiftLeft');
  await page.keyboard.down('KeyA');
  try { await expect(page.getByLabel('Round phase')).toHaveText('Round results', {timeout:60_000}); }
  finally { await page.keyboard.up('KeyA'); await page.keyboard.up('ShiftLeft'); }
  await expect(page.getByLabel('Round number')).toHaveText('Round 2', {timeout:8000});
  await expect(page.getByLabel('Blue score')).toHaveText('1');
});
