import { expect, test } from '@playwright/test';

test('a passive scene tuning draft cannot carry into a fresh standing scene', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  await page.getByLabel('Test scene').selectOption('ragdoll');
  await page.getByLabel('Gravity Y (m/s²)').fill('-1');
  await page.getByLabel('Test scene').selectOption('standing');
  await expect(page.getByLabel('Gravity Y (m/s²)')).toHaveValue('-9.81');
  await page.getByLabel('Solver iterations').fill('12');
  await page.getByRole('button', {name: 'Apply tuning'}).click();
  await expect(page.getByLabel('Gravity Y (m/s²)')).toHaveValue('-9.81');
  await expect(page.getByLabel('Solver iterations')).toHaveValue('12');
  await expect(page.getByLabel('Enable character control')).toBeChecked();
  const downloaded = page.waitForEvent('download');
  await page.getByRole('button', { name: 'Export configuration' }).click();
  let text = '';
  for await (const chunk of (await (await downloaded).createReadStream())!) text += chunk.toString();
  expect(JSON.parse(text)).toEqual({schema_version: 1, gravity_y: -9.81, solver_iterations: 12});
});
