import { expect, test } from '@playwright/test';
import { execFileSync } from 'node:child_process';

async function fixtureRecording(page: import('@playwright/test').Page, gravity: number) {
  return page.evaluate(async gravityY => {
    const { default: init, LocalSimulation } = await import(/* @vite-ignore */ String('/src/generated/wasm/brawler_wasm.js'));
    await init();
    const simulation = new LocalSimulation(JSON.stringify({ schema_version: 1, gravity_y: gravityY, solver_iterations: 8 }));
    simulation.step();
    const json = simulation.export_recording() as string;
    simulation.free();
    return json;
  }, gravity);
}

test('tuning and exported browser recordings replay in the native engine and visible page', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('button', { name: 'Apply tuning' })).toBeVisible();
  await page.getByLabel('Gravity Y (m/s²)').fill('-3');
  await page.getByLabel('Solver iterations').fill('12');
  await page.getByRole('button', { name: 'Apply tuning' }).click();
  await expect(page.locator('#tick')).toHaveText('0');
  await expect(page.getByRole('button', { name: 'Resume', exact: true })).toBeVisible();
  await page.getByRole('button', { name: 'Apply upward impulse' }).click();
  for (let i = 0; i < 3; i++) await page.getByRole('button', { name: 'Single step' }).click();
  const before = await page.locator('#height').textContent();
  const downloadPromise = page.waitForEvent('download');
  await page.getByRole('button', { name: 'Export recording' }).click();
  const download = await downloadPromise;
  let json = '';
  for await (const chunk of (await download.createReadStream())!) json += chunk.toString();
  const recording = JSON.parse(json);
  expect(recording.config.gravity_y).toBe(-3);
  expect(recording.commands).toEqual([{ tick: 1, impulse: [0, 4, 0] }]);
  const native = JSON.parse(execFileSync('cargo', [
    'run', '--quiet', '--locked', '--release', '-p', 'brawler-engine', '--example', 'replay_fixture',
  ], { cwd: '..', input: json, encoding: 'utf8' })) as number[][];
  expect(native).toHaveLength(4);
  expect(native[3]![1]!.toFixed(3) + ' m').toBe(before);
  await page.getByLabel('Import recording').setInputFiles({ name: 'fixture.json', mimeType: 'application/json', buffer: Buffer.from(json) });
  await expect(page.locator('#tick')).toHaveText('0');
  await expect(page.getByRole('button', { name: 'Apply upward impulse' })).toBeDisabled();
  await page.getByRole('button', { name: 'Resume', exact: true }).click();
  await expect(page.locator('#tick')).toHaveText('3');
  await expect(page.locator('#height')).toHaveText(before!);
  await expect(page.getByRole('button', { name: 'Single step' })).toBeDisabled();
  const invalid = JSON.stringify({ ...recording, engine_hash: 'incompatible' });
  await page.getByLabel('Import recording').setInputFiles({ name: 'bad.json', mimeType: 'application/json', buffer: Buffer.from(invalid) });
  await expect(page.getByRole('alert')).toContainText('different engine build');
  await expect(page.locator('#tick')).toHaveText('3');
});

test('nondefault config and impulses reproduce all 14 state fields across native and WASM', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  const browser = await page.evaluate(async () => {
    const { default: init, LocalSimulation } = await import(/* @vite-ignore */ String('/src/generated/wasm/brawler_wasm.js'));
    await init();
    const simulation = new LocalSimulation(JSON.stringify({ schema_version: 1, gravity_y: -7, solver_iterations: 12 }));
    const states = [Array.from(simulation.fixture_state() as Float32Array)];
    for (let tick = 1; tick <= 240; tick++) {
      if ([20, 90, 150].includes(tick)) simulation.queue_impulse(1, 4, 0);
      simulation.step();
      states.push(Array.from(simulation.fixture_state() as Float32Array));
    }
    const recording = simulation.export_recording() as string;
    simulation.free();
    return { recording, states };
  });
  const native = JSON.parse(execFileSync('cargo', [
    'run', '--quiet', '--locked', '--release', '-p', 'brawler-engine', '--example', 'replay_fixture',
  ], { cwd: '..', input: browser.recording, encoding: 'utf8' })) as number[][];
  expect(native).toHaveLength(browser.states.length);
  let maximumDeviation = 0;
  for (let tick = 0; tick < native.length; tick++) {
    expect(native[tick]).toHaveLength(14);
    for (let field = 0; field < 14; field++) {
      maximumDeviation = Math.max(maximumDeviation, Math.abs(native[tick]![field]! - browser.states[tick]![field]!));
    }
  }
  expect(maximumDeviation).toBeLessThan(1e-5);
});

test('a file read started before tuning cannot replace the newer fixture', async ({ page }) => {
  await page.addInitScript(() => {
    const original = File.prototype.text;
    File.prototype.text = function () {
      return new Promise<string>(resolve => {
        (window as unknown as { releaseFile: () => Promise<void> }).releaseFile = async () => {
          resolve(await original.call(this));
          await new Promise<void>(done => requestAnimationFrame(() => requestAnimationFrame(() => done())));
        };
      });
    };
  });
  await page.goto('/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  const json = await fixtureRecording(page, -7);
  await page.getByLabel('Import recording').setInputFiles({ name: 'delayed.json', mimeType: 'application/json', buffer: Buffer.from(json) });
  await page.waitForFunction(() => 'releaseFile' in window);
  await page.getByLabel('Gravity Y (m/s²)').fill('-3');
  await page.getByRole('button', { name: 'Apply tuning' }).click();
  await page.evaluate(async () => {
    await (window as unknown as { releaseFile: () => Promise<void> }).releaseFile();
  });
  await expect(page.getByLabel('Gravity Y (m/s²)')).toHaveValue('-3');
  await expect(page.locator('#replay-status')).toContainText('Recording completed ticks');
});

test('a solver-only tuning change preserves valid fractional imported gravity', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  const json = await fixtureRecording(page, -9.815);
  await page.getByLabel('Import recording').setInputFiles({ name: 'fractional.json', mimeType: 'application/json', buffer: Buffer.from(json) });
  await expect(page.locator('#replay-status')).toContainText('Replay loaded');
  await page.getByLabel('Solver iterations').fill('12');
  await page.getByRole('button', { name: 'Apply tuning' }).click();
  await expect(page.locator('#replay-status')).toContainText('Recording completed ticks');
  const downloaded = page.waitForEvent('download');
  await page.getByRole('button', { name: 'Export configuration' }).click();
  let text = '';
  for await (const chunk of (await (await downloaded).createReadStream())!) text += chunk.toString();
  expect(JSON.parse(text).gravity_y).toBeCloseTo(-9.815, 5);
  expect(JSON.parse(text).solver_iterations).toBe(12);
});
