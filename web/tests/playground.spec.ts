import { expect, test } from '@playwright/test';

test('pause, exact single step, bounded impulse, reset and collider overlay work', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  await page.getByRole('button', { name: 'Pause' }).click();
  await page.getByRole('button', { name: 'Reset fixture' }).click();
  await expect(page.locator('#tick')).toHaveText('0');
  await expect(page.locator('#height')).toHaveText('4.000 m');
  await page.getByRole('button', { name: 'Single step' }).click();
  await expect(page.locator('#tick')).toHaveText('1');
  await page.getByRole('button', { name: 'Apply upward impulse' }).click();
  await expect(page.getByRole('button', { name: 'Apply upward impulse' })).toBeDisabled();
  await page.getByRole('button', { name: 'Single step' }).click();
  await expect(page.locator('#tick')).toHaveText('2');
  await expect.poll(async () => Number.parseFloat((await page.locator('#vertical-speed').textContent())!))
    .toBeGreaterThan(3.5);
  const canvas = page.locator('canvas');
  const withoutOverlay = await canvas.screenshot();
  await page.getByRole('checkbox', { name: 'Show colliders' }).check();
  await expect(page.locator('#debug-segments')).toHaveText('24');
  const withOverlay = await canvas.screenshot();
  expect(withOverlay.equals(withoutOverlay)).toBe(false);
  await page.getByRole('checkbox', { name: 'Show colliders' }).uncheck();
  expect((await canvas.screenshot()).equals(withoutOverlay)).toBe(true);
  await page.getByRole('button', { name: 'Reset fixture' }).click();
  await expect(page.locator('#tick')).toHaveText('0');
  await page.getByRole('button', { name: 'Resume' }).click();
  await expect(page.getByRole('button', { name: 'Single step' })).toBeDisabled();
  await expect.poll(async () => Number(await page.locator('#tick').textContent())).toBeGreaterThan(10);
});

test('long diagnostics panel does not push the desktop scene below the viewport', async ({ page }) => {
  await page.setViewportSize({ width: 1440, height: 900 });
  await page.goto('/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  const canvas = await page.locator('canvas').boundingBox();
  expect(canvas!.y + canvas!.height).toBeLessThanOrEqual(900);
});

test('exported configuration is the versioned configuration used by the fixture', async ({ page }) => {
  await page.goto('/');
  const download = page.waitForEvent('download');
  await page.getByRole('button', { name: 'Export configuration' }).click();
  const file = await download;
  const stream = await file.createReadStream();
  let text = '';
  for await (const chunk of stream!) text += chunk.toString();
  expect(JSON.parse(text)).toEqual({ schema_version: 1, gravity_y: -9.81, solver_iterations: 8 });
});
