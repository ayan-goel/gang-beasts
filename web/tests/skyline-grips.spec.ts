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

test('a player catches the roof when the west walkway falls, then releases to a ring-out', async ({ page }) => {
  await page.goto('/play/');
  await expect(page.getByRole('button', { name: 'Start match' })).toBeEnabled();
  const result = await page.evaluate(async () => {
    const { LocalDuel } = await import(/* @vite-ignore */ String('/src/generated/wasm/brawler_wasm.js'));
    const game = LocalDuel.skyline_match(await (await fetch('/src/generated/physics.json')).text());
    try {
      for (let tick = 0; tick < 180; tick++) game.step();
      const parts: { part: string }[] = JSON.parse(game.definition_json()).bodies;
      const palmOffsets = ['left_hand', 'right_hand'].map(name => 14 * parts.findIndex(body => body.part === name));
      let caught = 0, consecutive = 0, maximumPalmDistance = 0;
      for (let tick = 0; tick < 7320; tick++) {
        if (tick === 6800) game.set_input(0, -1, 0, false);
        if (tick === 7050) game.set_input(0, 1, 0, false);
        if (tick === 7078) game.set_input(0, 0, 0, false);
        if (tick === 7160) { game.set_hand(0, 0, true); game.set_hand(0, 1, true); }
        game.step();
        const held = tick >= 7200 ? [0, 1].filter(hand => game.arena_grip(0, hand) === 1) : [];
        if (held.length) {
          const poses = game.body_states(0);
          for (const hand of held) {
            const offset = palmOffsets[hand]!;
            maximumPalmDistance = Math.max(maximumPalmDistance, Math.hypot(poses[offset] + 10, poses[offset + 1]));
          }
          if (!caught) caught = tick + 1;
          consecutive++;
        } else consecutive = 0;
      }
      const hanging = game.body_states(0)[2 * 14 + 1];
      game.clear_hands(0);
      for (let i = 0; i < 140; i++) game.step();
      return { caught, consecutive, maximumPalmDistance, hanging, match: JSON.parse(game.match_json()) };
    } finally { game.free(); }
  });
  expect(result.caught).toBe(7229);
  expect(result.consecutive).toBe(92);
  expect(result.maximumPalmDistance).toBeLessThan(0.18);
  expect(result.hanging).toBeLessThan(-0.4);
  expect(result.hanging).toBeGreaterThan(-1.5);
  expect(result.match.phase).toBe('round_results');
  expect(result.match.scores).toEqual([0, 1]);
});
