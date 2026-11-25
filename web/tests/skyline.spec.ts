import { expect, test } from '@playwright/test';

test('Skyline starts a local match, scores a physical ring-out and starts the next round', async ({ page }, testInfo) => {
  await page.goto('/play');
  await expect(page.getByRole('heading', { name: 'Skyline Melee' })).toBeVisible({ timeout: 2000 });
  await expect(page.getByRole('button', { name: 'Start match' })).toBeEnabled();
  await page.screenshot({ path: testInfo.outputPath('skyline-menu.png') });
  await page.getByRole('button', { name: 'Start match' }).click();
  await expect(page.getByLabel('Round phase')).toHaveText('Countdown');
  await expect(page.getByLabel('Round phase')).toHaveText('Playing', { timeout: 12000 });
  await page.screenshot({ path: testInfo.outputPath('skyline-round.png') });
  await page.keyboard.down('ShiftLeft');
  await page.keyboard.down('KeyA');
  await expect(page.getByLabel('Round phase')).toHaveText('Round results', { timeout: 30000 });
  await page.keyboard.up('KeyA');
  await page.keyboard.up('ShiftLeft');
  await expect(page.getByLabel('Blue score')).toHaveText('1');
  await expect(page.getByText('Blue wins the round')).toBeVisible();
  await expect(page.getByLabel('Round number')).toHaveText('Round 2', { timeout: 18000 });
  await expect(page.getByLabel('Blue score')).toHaveText('1');
  await expect(page.locator('canvas')).toHaveCount(1);
});

test('WASM detaches each real walkway at its deadline and rematch restores the original poses', async ({ page }) => {
  await page.goto('/play');
  await expect(page.getByRole('button', { name: 'Start match' })).toBeEnabled();
  const observations = await page.evaluate(async () => {
    const { LocalDuel } = await import(/* @vite-ignore */ String('/src/generated/wasm/brawler_wasm.js'));
    const game = LocalDuel.skyline_match(await (await fetch('/src/generated/physics.json')).text());
    try {
      const initial = Array.from<number>(game.arena_poses());
      const samples: { tick: number; hazard: string; west: number; east: number; westVisible: number }[] = [];
      for (let i = 0; i < 180; i++)game.step();
      for (let tick = 1; tick <= 9020; tick++) {
        game.step();
        if ([5399, 5400, 7200, 7220, 9000, 9020].includes(tick)) {
          const poses = game.arena_poses();
          samples.push({
            tick: Number(game.tick()), hazard: JSON.parse(game.match_json()).hazard,
            west: poses[7 * 8 + 1], east: poses[8 * 8 + 1], westVisible: poses[7 * 8 + 7]
          });
        }
      }
      game.reset();
      return { initial, samples, reset: Array.from<number>(game.arena_poses()), match: JSON.parse(game.match_json()) };
    } finally { game.free(); }
  });
  expect(observations.samples.map(s => [s.tick, s.hazard])).toEqual([
    [5399, 'quiet'], [5400, 'west_warning'], [7200, 'east_warning'], [7220, 'east_warning'], [9000, 'gust_warning'], [9020, 'gust_warning'],
  ]);
  expect(observations.samples[3]!.west).toBeLessThan(-0.5);
  expect(observations.samples[3]!.east).toBeCloseTo(-0.22);
  expect(observations.samples[5]!.east).toBeLessThan(-0.5);
  expect(observations.samples[5]!.westVisible).toBe(0);
  expect(observations.reset).toEqual(observations.initial);
  expect(observations.match.phase).toBe('countdown');
  expect(observations.match.scores).toEqual([0, 0]);
});

test('a match-winning round offers a rematch without a page reload', async ({ page }) => {
  await page.goto('/play/');
  await expect(page.getByRole('button', { name: 'Start match' })).toBeEnabled();
  await page.evaluate(async () => {
    const { LocalDuel } = await import(/* @vite-ignore */ String('/src/generated/wasm/brawler_wasm.js'));
    const step = LocalDuel.prototype.step;
    // Advance more ordinary fixed ticks per render so the test can play three full rounds.
    LocalDuel.prototype.step = function() {
      for (let i = 0; i < 5; i++) step.call(this);
    };
  });
  await page.getByRole('button', { name: 'Start match' }).click();
  for (let i = 1; i <= 3; i++) {
    await expect(page.getByLabel('Round number')).toHaveText(`Round ${i}`);
    await expect(page.getByLabel('Round phase')).toHaveText('Playing');
    await page.keyboard.down('ShiftLeft');
    await page.keyboard.down('KeyA');
    await expect(page.getByLabel('Blue score')).toHaveText(String(i), { timeout: 10000 });
    await page.keyboard.up('KeyA');
    await page.keyboard.up('ShiftLeft');
  }
  await expect(page.getByRole('heading', { name: 'Blue wins the match' })).toBeVisible();
  const nav = await page.evaluate(() => performance.timeOrigin);
  await page.getByRole('button', { name: 'Rematch' }).click();
  await expect(page.getByLabel('Blue score')).toHaveText('0');
  await expect(page.getByLabel('Round number')).toHaveText('Round 1');
  expect(await page.evaluate(() => performance.timeOrigin)).toBe(nav);
});

test('small screens keep the start button reachable and the roof visible', async ({ page }, testInfo) => {
  await page.setViewportSize({ width: 390, height: 844 });
  await page.goto('/play/');
  await expect(page.getByRole('button', { name: 'Start match' })).toBeEnabled();
  await expect(page.getByRole('button', { name: 'Start match' })).toBeInViewport();
  const canvas = await page.locator('canvas').boundingBox();
  expect(canvas!.height).toBeGreaterThan(170);
  expect(canvas!.width).toBeLessThanOrEqual(390);
  await page.screenshot({ path: testInfo.outputPath('skyline-narrow.png') });
});
