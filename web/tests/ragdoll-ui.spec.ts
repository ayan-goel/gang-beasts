import { expect, test } from '@playwright/test';

test('ragdoll scene supports inspection, impulses, tuning and a clean switch back', async ({ page }) => {
  const errors: string[] = [];
  page.on('pageerror', error => errors.push(error.message));
  await page.goto('/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  await page.getByLabel('Test scene').selectOption('ragdoll');
  await expect(page.locator('#tick')).toHaveText('0');
  await expect(page.getByRole('button', { name: 'Resume', exact: true })).toBeVisible();
  await expect(page.locator('#rig-summary')).toHaveText('15 bodies · 14 joints · 40.0 kg');
  await expect(page.getByRole('button', { name: 'Export recording' })).toHaveCount(0);
  await page.getByLabel('Inspect body').selectOption('0');
  await expect(page.locator('#height')).toHaveText('3.420 m');
  const withoutJoints = await page.locator('canvas').screenshot();
  await page.getByLabel('Show joint frames').check();
  await expect(page.locator('#joint-frame-count')).toHaveText('14');
  const plain = await page.locator('canvas').screenshot();
  expect(plain.equals(withoutJoints)).toBe(false);
  await page.getByLabel('Show colliders').check();
  await expect(page.locator('#debug-segments')).not.toHaveText('0');
  expect((await page.locator('canvas').screenshot()).equals(plain)).toBe(false);
  await page.getByRole('button', { name: 'Apply forward impulse' }).click();
  await expect(page.getByRole('button', { name: 'Apply forward impulse' })).toBeDisabled();
  await page.getByRole('button', { name: 'Single step' }).click();
  await expect(page.locator('#tick')).toHaveText('1');
  await expect(page.getByRole('button', { name: 'Apply forward impulse' })).toBeEnabled();
  await page.getByLabel('Show colliders').uncheck();
  await page.getByLabel('Show joint frames').uncheck();
  const beforeFall = await page.locator('canvas').screenshot();
  await page.getByRole('button', { name: 'Resume', exact: true }).click();
  await expect.poll(async () => Number(await page.locator('#tick').textContent())).toBeGreaterThan(90);
  await page.getByRole('button', { name: 'Pause', exact: true }).click();
  await expect(page.locator('#height')).not.toHaveText('3.420 m');
  expect((await page.locator('canvas').screenshot()).equals(beforeFall)).toBe(false);
  await page.getByRole('button', { name: 'Reset fixture' }).click();
  await expect(page.locator('#height')).toHaveText('3.420 m');
  await page.getByLabel('Gravity Y (m/s²)').fill('-3');
  await page.getByRole('button', { name: 'Apply tuning' }).click();
  await expect(page.locator('#tick')).toHaveText('0');
  await page.getByLabel('Test scene').selectOption('cube');
  await expect(page.getByRole('button', { name: 'Export recording' })).toBeVisible();
  await expect(page.getByLabel('Inspect body')).toHaveCount(0);
  await expect(page.locator('canvas')).toHaveCount(1);
  expect(errors).toEqual([]);
});

test('switching scenes releases the previous WebGL context', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  await page.evaluate(() => {
    const previous = document.querySelector('canvas')!.getContext('webgl2')!;
    (window as unknown as { previousContextLost: () => boolean }).previousContextLost = () => previous.isContextLost();
  });
  await page.getByLabel('Test scene').selectOption('ragdoll');
  await expect(page.locator('#rig-summary')).toBeVisible();
  await expect.poll(() => page.evaluate(() =>
    (window as unknown as { previousContextLost: () => boolean }).previousContextLost()), { timeout: 2000 }).toBe(true);
});
