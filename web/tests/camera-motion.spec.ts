import { expect, test, type Page } from '@playwright/test';
import type { Scene } from 'three';
import { observeArenaMeshes } from './support/arenaMeshes';
import { gripMatch } from './support/gripMatch';

async function firstTurn(page: Page) {
  await page.bringToFront();
  await page.locator('canvas').focus();
  const probe = await page.evaluateHandle(() => {
    const scene = window.skylineScenes.map(ref => ref.deref()).find(s => s?.getObjectByName('deck-1'))!;
    const previous = scene.onBeforeRender;
    const state = {initial: NaN, step: 0};
    scene.onBeforeRender = function(this: Scene, ...args) {
      previous.apply(this, args);
      const matrix = args[2].matrixWorld.elements;
      const yaw = Math.atan2(matrix[8], matrix[10]);
      if (!Number.isFinite(state.initial)) state.initial = yaw;
      const difference = yaw - state.initial;
      if (Math.abs(difference) > .00001) {
        state.step = Math.abs(Math.atan2(Math.sin(difference), Math.cos(difference)));
        scene.onBeforeRender = previous;
      }
    };
    return state;
  });
  try {
    await expect.poll(() => probe.evaluate(s => Number.isFinite(s.initial))).toBe(true);
    await page.keyboard.press('KeyX');
    await expect.poll(() => probe.evaluate(s => s.step)).toBeGreaterThan(0);
    return await probe.evaluate(s => s.step);
  } finally { await probe.dispose(); }
}

test('local camera honors device changes, a menu override and its saved choice on reload', async ({page}) => {
  await page.emulateMedia({reducedMotion:'reduce'});
  await observeArenaMeshes(page);
  await page.goto('/play/');
  for (const width of [320, 768, 1024]) {
    await page.setViewportSize({width, height:800});
    await expect(page.getByLabel('Camera motion')).toBeInViewport();
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  }
  await page.getByRole('button', {name:'Start match'}).click();
  await expect(page.getByLabel('Round phase')).toHaveText('Playing', {timeout:15000});
  expect(await firstTurn(page)).toBeCloseTo(Math.PI/4, 5);
  await page.emulateMedia({reducedMotion:'no-preference'});
  expect(await firstTurn(page)).toBeLessThan(Math.PI/4 - .01);
  await page.keyboard.press('Escape');
  const menu = page.getByRole('dialog');
  await menu.getByLabel('Camera motion').selectOption('reduced');
  await menu.getByRole('button', {name:'Resume match'}).click();
  await page.reload();
  await expect(page.getByLabel('Camera motion')).toHaveValue('reduced');
  await page.getByRole('button', {name:'Start match'}).click();
  await expect(page.getByLabel('Round phase')).toHaveText('Playing', {timeout:15000});
  expect(await firstTurn(page)).toBeCloseTo(Math.PI/4, 5);
});

test('two private players can use different camera motion in the same round', async ({page:host,browser}) => {
  await gripMatch(host,browser,async (guest,start) => {
    for (const player of [host,guest]) {
      await player.emulateMedia({reducedMotion:'no-preference'});
      await observeArenaMeshes(player);
    }
    await start();
    await host.emulateMedia({reducedMotion:'reduce'});
    expect(await firstTurn(host)).toBeCloseTo(Math.PI/4, 5);
    await expect(guest.getByLabel('Camera motion')).toHaveCount(0);
    expect(await firstTurn(guest)).toBeLessThan(Math.PI/4 - .01);
    for (const player of [host,guest]) await expect(player.getByLabel('Round phase')).toHaveText('Playing');
  });
});
