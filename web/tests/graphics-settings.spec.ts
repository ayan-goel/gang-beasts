import { expect, test } from '@playwright/test';
import type { DirectionalLight } from 'three';
import { observeArenaMeshes } from './support/arenaMeshes';
import { graphicsQuality } from './support/graphicsQuality';
import { gripMatch } from './support/gripMatch';

test.use({ viewport:{width:800,height:650}, deviceScaleFactor:2 });
if (process.platform === 'darwin') test.use({launchOptions:{args:['--use-gl=angle','--use-angle=metal']}});

test('changing graphics replaces the shadow target in place and retains the choice after reload', async ({page}, info) => {
  await observeArenaMeshes(page);
  await page.goto('/play/');
  const setting = page.getByRole('combobox', {name:'Graphics'});
  await page.setViewportSize({width:390,height:800});
  await expect(setting).toBeInViewport();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
  await page.screenshot({path:info.outputPath('local-graphics-portrait.png')});
  await page.setViewportSize({width:800,height:650});
  await setting.selectOption('medium');
  await expect.poll(() => graphicsQuality(page)).toEqual({ratio:1.5, shadow:1024});
  await page.getByRole('button', {name:'Start match'}).click();
  await expect(page.getByLabel('Round phase')).toHaveText('Playing', {timeout:12000});
  const clock = page.locator('.match-round strong');
  const initialTime = await clock.innerText();
  const canvas = await page.locator('canvas').elementHandle();
  const retired = await page.evaluateHandle(() => {
    const state = {count:0};
    const scene = window.skylineScenes.map(ref => ref.deref()).find(s => s?.getObjectByName('deck-1'))!;
    const sun = scene.children.find(s => s.type === 'DirectionalLight') as DirectionalLight;
    sun.shadow.map!.addEventListener('dispose', () => state.count++);
    return state;
  });
  await setting.selectOption('low');
  await expect.poll(() => graphicsQuality(page)).toEqual({ratio:.75, shadow:512});
  expect(await retired.evaluate(state => state.count)).toBe(1);
  await setting.selectOption('high');
  await expect.poll(() => graphicsQuality(page)).toEqual({ratio:2, shadow:2048});
  expect(await canvas!.evaluate(node => node === document.querySelector('canvas') && !(node as HTMLCanvasElement).getContext('webgl2')!.isContextLost())).toBe(true);
  await expect(page.getByLabel('Round phase')).toHaveText('Playing');
  await expect(page.getByLabel('Round number')).toHaveText('Round 1');
  await expect(clock).not.toHaveText(initialTime, {timeout:6000});
  for (const [preset, shadow] of [['medium',1024], ['low',512], ['high',2048]] as const) {
    await setting.selectOption(preset);
    await expect.poll(async () => (await graphicsQuality(page))?.shadow).toBe(shadow);
  }
  await retired.dispose();
  await canvas!.dispose();
  await page.reload();
  await expect(setting).toHaveValue('high');
  await expect.poll(() => graphicsQuality(page)).toEqual({ratio:2, shadow:2048});
  const lobby = await page.context().newPage();
  try {
    await lobby.setViewportSize({width:390, height:800});
    await lobby.goto('/rooms/');
    const other = lobby.getByRole('combobox', {name:'Graphics'});
    await expect(other).toHaveValue('high');
    await expect(other).toBeInViewport();
    expect(await lobby.evaluate(() => document.documentElement.scrollWidth <= innerWidth)).toBe(true);
    await other.selectOption('low');
    await page.bringToFront();
    await expect(setting).toHaveValue('low');
    await expect.poll(() => graphicsQuality(page)).toEqual({ratio:.75, shadow:512});
    await setting.selectOption('auto');
    await expect(other).toHaveValue('auto');
  } finally { await lobby.close(); }
});

test('a private player can lower graphics while the other seat keeps playing at its own quality', async ({page: host,browser}) => {
  await gripMatch(host,browser,async (guest,start) => {
    for (const p of [host,guest]) await observeArenaMeshes(p);
    await start();
    await host.getByRole('combobox', {name:'Graphics'}).selectOption('low');
    await expect.poll(() => graphicsQuality(host)).toEqual({ratio:.75,shadow:512});
    await expect(guest.getByRole('combobox', {name:'Graphics'})).toHaveValue('auto');
    await guest.getByRole('combobox', {name:'Graphics'}).selectOption('high');
    await expect.poll(async () => (await graphicsQuality(guest))?.shadow).toBe(2048);
    await expect.poll(() => graphicsQuality(host)).toEqual({ratio:.75,shadow:512});
    await guest.bringToFront();
    await guest.locator('canvas').focus();
    await guest.keyboard.down('ShiftLeft');
    await guest.keyboard.down('KeyD');
    try {
      await Promise.all([host,guest].map(p => expect(p.getByLabel('Mint score')).toHaveText('1', {timeout:25000})));
    } finally { await guest.keyboard.up('KeyD'); await guest.keyboard.up('ShiftLeft'); }
  });
});
