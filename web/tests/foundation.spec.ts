import { expect, test } from '@playwright/test';
import { execFileSync } from 'node:child_process';

test('browser WASM reproduces the native falling-body trace', async ({ page }) => {
  const failures: string[] = [];
  page.on('pageerror', error => failures.push(error.message));
  page.on('console', message => {
    if (message.type() === 'error') failures.push(message.text());
  });
  const native = JSON.parse(execFileSync('cargo', [
    'run', '--quiet', '--locked', '--release', '-p', 'brawler-engine', '--example', 'foundation_trace',
  ], { cwd: '..', encoding: 'utf8' })) as number[][];
  await page.goto('/playground/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  await expect.poll(async () => Number(await page.locator('#tick').textContent())).toBeGreaterThan(90);
  await expect.poll(async () => Number.parseFloat((await page.locator('#height').textContent())!))
    .toBeCloseTo(0.5, 2);
  const browser = await page.evaluate(async () => {
    const { default: init, LocalSimulation } = await import(/* @vite-ignore */ String('/src/generated/wasm/brawler_wasm.js'));
    await init();
    const configuration = await (await fetch('/src/generated/physics.json')).text();
    const simulation = new LocalSimulation(configuration);
    const trace = [Array.from(simulation.position() as Float32Array)];
    for (let i = 0; i < 600; i++) {
      simulation.step();
      trace.push(Array.from(simulation.position() as Float32Array));
    }
    simulation.free();
    return trace;
  });
  expect(browser).toHaveLength(native.length);
  for (let tick = 0; tick < native.length; tick++) {
    for (let axis = 0; axis < 3; axis++) {
      expect(Math.abs(browser[tick]![axis]! - native[tick]![axis]!)).toBeLessThan(1e-5);
    }
  }
  expect(failures).toEqual([]);
  await expect(page.locator('canvas')).toBeVisible();
});

test('canvas stays inside its viewport while resizing', async ({ page }) => {
  await page.goto('/playground/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  // Shrink the host synchronously, before ResizeObserver can resize the GPU
  // buffer, to reproduce the one-frame overflow observed during window resize.
  const overflow = await page.evaluate(() => {
    const host = document.querySelector<HTMLElement>('.viewport')!;
    host.style.width = '200px';
    return host.querySelector('canvas')!.getBoundingClientRect().width > host.clientWidth;
  });
  expect(overflow).toBe(false);
});
