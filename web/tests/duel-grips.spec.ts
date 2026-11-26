import { expect, test } from '@playwright/test';

test('a held sparring hand identifies its target and clears on key release or focus loss', async ({ page }) => {
  await page.goto('/playground/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  await page.getByLabel('Test scene').selectOption('duel');
  const grips = page.getByLabel('Coral grips', { exact: true });
  await expect(grips).toHaveText('Hands free', { timeout: 2000 });
  for (const release of ['key', 'blur']) {
    await page.getByRole('button', { name: 'Resume', exact: true }).click();
    await expect.poll(async () => Number(await page.locator('#duel-tick').innerText())).toBeGreaterThan(59);
    await page.keyboard.down('j');
    await expect(grips).toContainText(/Left hand on Blue’s/);
    await expect.poll(async () => Number(await page.getByLabel('Coral stamina').textContent())).toBeLessThan(100);
    await expect(page.getByLabel('Blue stamina')).toHaveText('100');
    await page.keyboard.press('k');
    await expect(grips).toContainText(/Left hand on Blue’s/);
    if (release === 'blur') await page.getByLabel('Test scene').focus();
    else await page.keyboard.up('j');
    await expect(grips).toHaveText('Hands free');
    await page.keyboard.up('j');
    await expect(page.getByLabel('Coral stamina')).toHaveText('100');
    await page.getByRole('button', { name: 'Reset encounter' }).click();
  }
});

test('shared keyboard lift reaches the correct fighter and blur releases both', async ({ page }) => {
  await page.goto('/playground/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  await page.evaluate(async () => {
    const { LocalDuel } = await import(/* @vite-ignore */ String('/src/generated/wasm/brawler_wasm.js'));
    const original = LocalDuel.prototype.set_lift;
    const held = [false, false];
    Object.assign(window, { liftInputs: held });
    LocalDuel.prototype.set_lift = function (who: number, down: boolean) {
      held[who] = down;
      return original.call(this, who, down);
    };
  });
  await page.getByLabel('Test scene').selectOption('duel');
  await page.getByRole('button', { name: 'Resume', exact: true }).click();
  const held = () => page.evaluate(() => (window as unknown as { liftInputs: boolean[] }).liftInputs);
  await page.keyboard.down('Slash');
  await expect.poll(held).toEqual([false, true]);
  await page.keyboard.down('KeyE');
  await expect.poll(held).toEqual([true, true]);
  await page.getByLabel('Test scene').focus();
  await expect.poll(held).toEqual([false, false]);
  await page.keyboard.up('KeyE');
  await page.keyboard.up('Slash');
});
