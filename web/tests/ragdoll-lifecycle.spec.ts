import { expect, test } from '@playwright/test';

for (const stage of ['render', 'constructor'] as const) {
  test(`failed ${stage} cleans up the native rig, canvas and GPU context`, async ({ page }) => {
    await page.goto('/');
    await expect(page.getByRole('status')).toHaveText('Simulation ready');
    await page.evaluate(async stage => {
      const loaded = performance.getEntriesByType('resource').find(entry => new URL(entry.name).pathname === '/src/generated/wasm/brawler_wasm.js')!;
      const { LocalRagdoll } = await import(/* @vite-ignore */ loaded.name);
      let freed = 0;
      let context: WebGL2RenderingContext | null = null;
      const originalFree = LocalRagdoll.prototype.free;
      LocalRagdoll.prototype.free = function(this: { free: () => void }) { freed++; originalFree.call(this); };
      const capture = () => { context = document.querySelector('canvas')!.getContext('webgl2'); };
      if (stage === 'render') {
        const original = LocalRagdoll.prototype.body_states;
        LocalRagdoll.prototype.body_states = function() {
          capture(); LocalRagdoll.prototype.body_states = original;
          return new Float32Array();
        };
      } else {
        const Original = window.ResizeObserver;
        window.ResizeObserver = class extends Original {
          constructor(callback: ResizeObserverCallback) {
            super(callback);
            capture(); window.ResizeObserver = Original;
            throw new Error('Injected observer setup failure');
          }
        };
      }
      (window as unknown as { failedResources: () => { freed: number; lost: boolean } }).failedResources =
        () => ({ freed, lost: context?.isContextLost() ?? false });
    }, stage);
    await page.getByLabel('Test scene').selectOption('ragdoll');
    await expect(page.getByRole('alert')).toContainText('Unable to start');
    await expect.poll(() => page.evaluate(() =>
      (window as unknown as { failedResources: () => { freed: number; lost: boolean } }).failedResources()),
    { timeout: 2000 }).toEqual({ freed: 1, lost: true });
    await expect(page.locator('canvas')).toHaveCount(0);
    await page.getByLabel('Test scene').selectOption('cube');
    await expect(page.getByRole('status')).toHaveText('Simulation ready');
    await expect(page.locator('canvas')).toHaveCount(1);
  });
}

test('playground routes the selected limb and forward impulse to the native rig', async ({ page }) => {
  await page.goto('/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  const result = await page.evaluate(async () => {
    const moduleUrl = (path: string) => performance.getEntriesByType('resource').find(entry => new URL(entry.name).pathname === path)!.name;
    const { LocalRagdoll } = await import(/* @vite-ignore */ moduleUrl('/src/generated/wasm/brawler_wasm.js'));
    const { RagdollSimulation } = await import(/* @vite-ignore */ moduleUrl('/src/game/RagdollSimulation.ts'));
    const { Playground } = await import(/* @vite-ignore */ moduleUrl('/src/game/Playground.ts'));
    const config = await (await fetch('/src/generated/physics.json')).text();
    const raw = new LocalRagdoll(config);
    const adapter = new RagdollSimulation(config);
    const playground = new Playground(adapter);
    playground.setPaused(true);
    adapter.selectBody(7);
    playground.queueImpulse([0, 0, -4]); playground.singleStep();
    raw.queue_impulse(7, 0, 0, -4); raw.step();
    const actual = Array.from(adapter.bodyStates());
    const expected = Array.from(raw.body_states());
    raw.free(); adapter.free();
    return { actual, expected };
  });
  expect(result.actual).toEqual(result.expected);
});
