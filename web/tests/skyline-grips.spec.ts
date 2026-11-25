import { expect, test } from '@playwright/test';

test('Coral can grab a rooftop crate with the keyboard and release it during a lift', async ({ page }) => {
  await page.goto('/play/');
  await page.getByRole('button', { name: 'Start match' }).click();
  await expect(page.getByLabel('Round phase')).toHaveText('Playing', { timeout: 14000 });
  const grips = page.getByLabel('Coral grips');
  await page.keyboard.down('KeyS');
  await page.keyboard.down('KeyJ');
  await page.keyboard.down('KeyK');
  await expect(grips).toContainText('crate', { timeout: 12000 });
  await page.keyboard.up('KeyS');
  await page.keyboard.down('KeyE');
  await expect(grips).toContainText('crate');
  await page.keyboard.up('KeyJ');
  await page.keyboard.up('KeyK');
  await page.keyboard.up('KeyE');
  await expect(grips).toHaveCount(0);
});
