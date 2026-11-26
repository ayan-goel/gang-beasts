import { expect, test } from '@playwright/test';

test('practice gives Coral control before collapse and retries a missed rescue', async ({ page }) => {
  await page.goto('/playground/?scene=ledge');
  const start = page.getByRole('button', { name: 'Start rescue' });
  const result = page.getByLabel('Rescue status');
  await expect(start).toBeEnabled({timeout: 20000});
  await start.click();
  await page.keyboard.down('KeyJ');
  await page.keyboard.down('KeyK');
  await expect(result).toHaveText('Holding the roof edge', {timeout: 10000});
  await page.keyboard.down('Space');
  await expect(result).toHaveText('Back on the roof', {timeout: 15000});
  await page.keyboard.up('Space');
  await page.keyboard.up('KeyK');
  await page.keyboard.up('KeyJ');
  await page.getByRole('button', { name: 'Retry rescue' }).click();
  await start.click({timeout: 20000});
  await expect(result).toHaveText('Fell from the walkway', {timeout: 12000});
  await page.getByRole('button', { name: 'Retry rescue' }).click();
  await page.getByLabel('Test scene').selectOption('cube');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  await expect(page.locator('canvas')).toHaveCount(1);
});

test('a hanging player can release both hands and fall', async ({ page }) => {
  await page.goto('/playground/');
  await page.getByLabel('Test scene').selectOption('ledge');
  await page.getByRole('button', { name: 'Start rescue' }).click({timeout: 20000});
  const result = page.getByLabel('Rescue status');
  await page.keyboard.down('KeyJ');
  await page.keyboard.down('KeyK');
  await expect(result).toHaveText('Holding the roof edge', {timeout: 10000});
  await page.keyboard.up('KeyJ');
  await page.keyboard.up('KeyK');
  await expect(result).toHaveText('Fell from the walkway', {timeout: 10000});
});
