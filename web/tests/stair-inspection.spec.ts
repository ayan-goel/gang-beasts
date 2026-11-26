import { expect, test } from '@playwright/test';
import { execFileSync } from 'node:child_process';

test('stair inspection reproduces selected native frames and returns control at that pose', async ({ page }) => {
  await page.goto('/playground/?scene=stairs');
  await expect(page.getByLabel('Test scene')).toHaveValue('stairs');
  const settle = page.getByLabel('Settling ticks');
  const elapsed = page.getByLabel('Ticks after movement starts');
  const inspect = page.getByRole('button', { name: 'Inspect frame' });
  await expect(settle).toBeVisible({ timeout: 3000 });
  await page.getByLabel('Inspect body').selectOption({ label: 'head' });

  for (const [start, samples] of [[25, [81, 80, 81]], [60, [0, 240, 241, 420]]] as const) {
    const frames: number[][] = JSON.parse(execFileSync('cargo', [
      'run', '--quiet', '--locked', '--release', '-p', 'brawler-engine', '--example', 'stair_trace', '--', '--landing', String(start),
    ], { cwd: '..', encoding: 'utf8', maxBuffer: 4 * 1024 * 1024 }));
    await settle.fill(String(start));
    for (const sample of samples) {
      await elapsed.fill(String(sample));
      await inspect.click();
      const tick = start + sample;
      await expect(page.locator('#tick')).toHaveText(String(tick));
      await expect(page.locator('#height')).toHaveText(frames[tick]![1]!.toFixed(3) + ' m');
      await expect(page.locator('#position-z')).toHaveText(frames[tick]![30]!.toFixed(2));
      await expect(page.getByRole('button', { name: 'Resume', exact: true })).toBeVisible();
    }
  }
  await elapsed.fill('421');
  await expect(inspect).toBeDisabled();
  await page.getByRole('button', { name: 'Single step' }).click();
  await expect(page.locator('#tick')).toHaveText('481');
  await page.getByRole('button', { name: 'Reset fixture' }).click();
  await expect(page.locator('#tick')).toHaveText('0');
});
