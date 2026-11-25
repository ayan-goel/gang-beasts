import { expect, test } from '@playwright/test';

test('two players land independent punches in the sparring scene and reset the encounter', async ({page}, testInfo) => {
  await page.goto('/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  await expect(page.locator('option[value="duel"]')).toHaveText('Local sparring');
  await page.getByLabel('Test scene').selectOption('duel');
  await expect(page.locator('canvas')).toHaveAttribute('aria-label', /two fighters/i);
  const start = page.getByRole('button', { name: 'Resume', exact: true });
  await start.click();
  await expect.poll(async () => Number(await page.locator('#duel-tick').innerText())).toBeGreaterThan(59);
  await page.keyboard.press('j');
  await expect(page.locator('#duel-hit')).toContainText('Coral');
  await expect(page.locator('#coral-hits')).toHaveText('1');
  await expect(page.locator('#blue-hits')).toHaveText('0');
  await page.locator('.viewport').screenshot({ path: testInfo.outputPath('duel.png') });
  await page.getByRole('button', { name: 'Reset encounter' }).click();
  await expect(page.locator('#duel-tick')).toHaveText('0');
  await expect(page.locator('#coral-hits')).toHaveText('0');
  await start.click();
  await expect.poll(async () => Number(await page.locator('#duel-tick').innerText())).toBeGreaterThan(59);
  await page.keyboard.press('Comma');
  await expect(page.locator('#blue-hits')).toHaveText('1');
  await page.getByLabel('Test scene').selectOption('cube');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  await expect(page.locator('canvas')).toHaveCount(1);
});

test('native and WASM agree on both rigs, reciprocal grips and hit events at each completed tick', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  const result = await page.evaluate(async () => {
    const { default: init, LocalDuel } = await import(/* @vite-ignore */ String('/src/generated/wasm/brawler_wasm.js'));
    await init();
    const duel = new LocalDuel(await (await fetch('/src/generated/physics.json')).text());
    try {
      let rejected = 0;
      for (const args of [[256, 0, true], [0, '0', true], [1, 0, 'false']]) {
        try { duel.set_hand(...args); } catch { rejected++; }
      }
      const frames: number[][][] = [];
      const hits: (string | number)[][] = [];
      const stamina: number[][] = [];
      const grips: (number | null)[][][] = [];
      const saved = duel.body_states(0);
      for (let tick = 0; tick <= 240; tick++) {
        if (tick === 61) for (const id of [0, 1]) {
          duel.set_hand(id, 0, true);
          duel.set_hand(id, 0, false);
        }
        if (tick === 92) {
          duel.set_input(1, -1, 0, false);
          duel.set_jump(0, true);
        }
        if (tick === 131) {
          duel.reset();
          duel.set_input(1, 0, 1, false);
          for (const id of [0, 1]) duel.set_hand(id, 0, true);
        }
        if (tick === 156) duel.set_input(1, 0, 0, false);
        if (tick === 205) duel.clear_hands(0);
        if (tick === 220) duel.set_hand(1, 0, false);
        if (tick) duel.step();
        stamina.push([duel.stamina(0), duel.stamina(1)]);
        grips.push([0, 1].map(id => [0, 1].map(hand => duel.grip_region(id, hand) ?? null)));
        frames.push([Array.from(duel.body_states(0)), Array.from(duel.body_states(1))]);
        for (const h of JSON.parse(duel.hits_json() ?? '[]')) {
          hits.push([h.tick, h.attacker, h.defender, h.attackId, h.hand, h.region, h.damage, h.energyJ, h.closingSpeed]);
        }
      }
      return { frames, hits, grips, stamina, rejected, saved: Array.from<number>(saved) };
    } finally { duel.free(); }
  });
  const { execFileSync } = await import('node:child_process');
  const native: { stamina: number[][]; grips: (number | null)[][][]; frames: number[][][]; hits: (string | number)[][] } = JSON.parse(execFileSync('cargo',
    ['run', '--release', '--quiet', '--locked', '-p', 'brawler-engine', '--example', 'duel_trace'],
    { cwd: '..', encoding: 'utf8', maxBuffer: 4*1024*1024 }));
  expect(result.rejected).toBe(3);
  expect(result.saved).toEqual(result.frames[0]![0]);
  expect(result.grips).toEqual(native.grips);
  expect(native.stamina).toHaveLength(result.stamina.length);
  for (const [tick, players] of result.stamina.entries()) {
    for (const [id, value] of players.entries()) expect(value).toBeCloseTo(native.stamina[tick]![id]!, 4);
  }
  expect(result.grips[200]!.every(player => player[0] != null)).toBe(true);
  expect(result.grips[205]![0]![0]).toBeNull();
  expect(result.grips[205]![1]![0]).not.toBeNull();
  expect(result.grips[220]).toEqual([[null, null], [null, null]]);
  expect(result.hits.length).toBeGreaterThanOrEqual(2);
  expect(result.hits).toHaveLength(native.hits.length);
  for (const [i, hit] of result.hits.entries()) {
    expect(hit.slice(0, 6)).toEqual(native.hits[i]!.slice(0, 6));
    for (let j = 6; j < 9; j++) expect(hit[j]).toBeCloseTo(Number(native.hits[i]![j]), 5);
  }
  let maxDifference = 0;
  expect(native.frames).toHaveLength(241);
  for (const [tick, fighters] of native.frames.entries()) {
    for (const [id, body] of fighters.entries()) {
      const frame = result.frames[tick]![id]!;
      expect(frame).toHaveLength(210);
      for (const [j, value] of body.entries()) maxDifference = Math.max(maxDifference, Math.abs(value - frame[j]!));
    }
  }
  expect(maxDifference).toBeLessThan(1e-5);
});

test('shared keyboard routes directions to separate fighters and blur clears both', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  await page.evaluate(async () => {
    const { LocalDuel } = await import(/* @vite-ignore */ String('/src/generated/wasm/brawler_wasm.js'));
    const original = LocalDuel.prototype.set_input;
    const commands: unknown[][] = [];
    (window as unknown as { duelCommands: unknown[][] }).duelCommands = commands;
    LocalDuel.prototype.set_input = function (...args: unknown[]) {
      commands.push(args);
      return original.apply(this, args);
    };
  });
  await page.getByLabel('Test scene').selectOption('duel');
  await page.getByRole('button', { name: 'Resume', exact: true }).click();
  const last = () => page.evaluate(() => {
    const commands = (window as unknown as { duelCommands: number[][] }).duelCommands;
    return [0, 1].map(id => commands.filter(command => command[0] === id).at(-1));
  });
  await page.keyboard.down('w');
  await page.keyboard.down('ArrowRight');
  await expect.poll(last).toEqual([[0, 0, -1, false], [1, 1, 0, false]]);
  await page.keyboard.down('ShiftRight');
  await expect.poll(last).toEqual([[0, 0, -1, false], [1, 1, 0, true]]);
  await page.getByLabel('Test scene').focus();
  await expect.poll(last).toEqual([[0, 0, 0, false], [1, 0, 0, false]]);
  await page.keyboard.up('ShiftRight');
  await page.keyboard.up('ArrowRight');
  await page.keyboard.up('w');
});
