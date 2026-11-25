import { expect, test } from '@playwright/test';

test('mouse chords and keyboard alternatives independently punch, hold and release the two hands', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  await page.getByLabel('Test scene').selectOption('locomotion');
  await expect(page.getByRole('button', { name: 'Left hand', exact: true })).toBeDisabled();
  await page.getByRole('button', { name: 'Resume', exact: true }).click();
  await page.locator('canvas').hover();
  await page.mouse.down({ button: 'left' });
  await page.mouse.down({ button: 'right' });
  await expect(page.locator('#left-hand-phase')).toHaveText('Reaching');
  await expect(page.locator('#right-hand-phase')).toHaveText('Reaching');
  await page.keyboard.down('j');
  await page.mouse.up({ button: 'left' });
  await expect(page.locator('#left-hand-phase')).toHaveText('Reaching');
  await page.keyboard.up('j');
  await expect(page.locator('#left-hand-phase')).toHaveText('Idle');
  await expect(page.locator('#right-hand-phase')).toHaveText('Reaching');
  await page.mouse.up({ button: 'right' });
  await expect(page.locator('#right-hand-phase')).toHaveText('Idle');
  await page.keyboard.down('k');
  await expect(page.locator('#right-hand-phase')).toHaveText('Reaching');
  await page.getByLabel('Gravity Y (m/s²)').focus();
  await expect(page.locator('#right-hand-phase')).toHaveText('Idle');
  await page.keyboard.up('k');
  const left = page.getByRole('button', { name: 'Left hand', exact: true });
  await left.hover();
  await page.mouse.down();
  await expect(page.locator('#left-hand-phase')).toHaveText('Reaching');
  await page.mouse.up();
  await expect(page.locator('#left-hand-phase')).toHaveText('Idle');
});

test('WASM tap moves the correct limb, enforces cooldown and retains owned hand records', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  const result = await page.evaluate(async () => {
    const { default: init, LocalRagdoll } = await import(/* @vite-ignore */ String('/src/generated/wasm/brawler_wasm.js'));
    await init();
    const config = await fetch('/src/generated/physics.json');
    const rig = LocalRagdoll.locomotion(await config.text());
    try {
      for (let tick = 0; tick < 60; tick++) rig.step();
      let rejected = 0;
      for (const args of [[2, true], [256, true], [-1, true], [0, 'true'], [null, true]]) {
        try { rig.set_hand(...args); } catch { rejected++; }
      }

      const leftHandZ = 7 * 14 + 2;
      const startZ = rig.body_states()[leftHandZ];
      rig.set_hand(0, true);
      rig.set_hand(0, false);
      const phases: number[] = [];
      let firstRecord: BigUint64Array | undefined;
      let farthest = 0;
      for (let tick = 1; tick <= 25; tick++) {
        rig.step();
        const hands = rig.hand_states();
        if (tick === 1) firstRecord = hands;
        phases.push(Number(hands[0]));
        farthest = Math.max(farthest, startZ - rig.body_states()[leftHandZ]);
        if (tick === 7) {
          rig.set_hand(0, true);
          rig.set_hand(0, false);
        }
      }
      return {
        phases, farthest, rejected,
        first: Array.from(firstRecord!, Number),
        last: Array.from(rig.hand_states(), Number),
      };
    } finally { rig.free(); }
  });
  expect(result.rejected).toBe(5);
  expect(result.phases).toEqual([...Array(5).fill(1), ...Array(7).fill(2), ...Array(12).fill(3), 0]);
  expect(result.farthest).toBeGreaterThan(0.38);
  expect(result.first).toEqual([1, 1, 0, 0]);
  expect(result.last).toEqual([0, 0, 0, 0]);
});

test('a two-hand reach remains supported for twenty seconds and either hand can retract', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  const result = await page.evaluate(async () => {
    const { default: init, LocalRagdoll } = await import(/* @vite-ignore */ String('/src/generated/wasm/brawler_wasm.js'));
    await init();
    const config = await fetch('/src/generated/physics.json');
    const rig = LocalRagdoll.locomotion(await config.text());
    try {
      for (let tick = 0; tick < 60; tick++) rig.step();
      rig.set_hand(0, true);
      rig.set_hand(1, true);
      let minChestHeight = Infinity;
      let uninterrupted = true;
      for (let tick = 1; tick <= 1200; tick++) {
        rig.step();
        minChestHeight = Math.min(minChestHeight, rig.body_states()[14 + 1]);
        if (tick >= 25) {
          const hands = rig.hand_states();
          uninterrupted &&= hands[0] === 4n && hands[2] === 4n;
        }
      }
      rig.set_hand(0, false);
      rig.step();
      return { minChestHeight, uninterrupted, final: Array.from(rig.hand_states(), Number) };
    } finally { rig.free(); }
  });
  expect(result.minChestHeight).toBeGreaterThan(0.95);
  expect(result.uninterrupted).toBe(true);
  expect(result.final).toEqual([0, 0, 4, 2]);
});
