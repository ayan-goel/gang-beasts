import { expect, test } from '@playwright/test';

test('a held sparring hand identifies its target and clears on key release or focus loss', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  await page.getByLabel('Test scene').selectOption('duel');
  const grips = page.getByLabel('Coral grips', { exact: true });
  await expect(grips).toHaveText('Hands free', { timeout: 2000 });
  for (const release of ['key', 'blur']) {
    await page.getByRole('button', { name: 'Resume', exact: true }).click();
    await expect.poll(async () => Number(await page.locator('#duel-tick').innerText())).toBeGreaterThan(59);
    await page.keyboard.down('j');
    await expect(grips).toContainText(/Left hand on Blue’s/);
    await page.keyboard.press('k');
    await expect(grips).toContainText(/Left hand on Blue’s/);
    if (release === 'blur') await page.getByLabel('Test scene').focus();
    else await page.keyboard.up('j');
    await expect(grips).toHaveText('Hands free');
    await page.keyboard.up('j');
    await page.getByRole('button', { name: 'Reset encounter' }).click();
  }
});
