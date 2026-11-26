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

  const duration = page.getByLabel('Ticks holding forward');
  await expect(duration).toHaveValue('240');
  for (const [start, walk, samples] of [
    [25, 403, [81, 80, 81, 119, 235, 403, 583]],
    [162, 628, [78, 256, 628, 808]],
    [60, 240, [0, 240, 241, 420]],
  ] as const) {
    const frames: number[][] = JSON.parse(execFileSync('cargo', [
      'run', '--quiet', '--locked', '--release', '-p', 'brawler-engine', '--example', 'stair_trace', '--', '--landing', String(start), String(walk),
    ], { cwd: '..', encoding: 'utf8', maxBuffer: 4 * 1024 * 1024 }));
    await settle.fill(String(start));
    await duration.fill(String(walk));
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
  await duration.fill('661');
  await expect(inspect).toBeDisabled();
  await duration.fill('240');
  await elapsed.fill('421');
  await expect(inspect).toBeDisabled();
  await page.getByRole('button', { name: 'Single step' }).click();
  await expect(page.locator('#tick')).toHaveText('481');
  await page.getByRole('button', { name: 'Reset fixture' }).click();
  await expect(page.locator('#tick')).toHaveText('0');
});
