import { expect, test } from '@playwright/test';
import { observeArenaMeshes } from './support/arenaMeshes';
import { graphicsQuality } from './support/graphicsQuality';

test.use({deviceScaleFactor:2, launchOptions:{args:['--use-gl=angle','--use-angle=swiftshader','--enable-unsafe-swiftshader']}});

test('automatic quality restores the low preset after a manual override', async ({page}) => {
  await observeArenaMeshes(page);
  await page.goto('/play/');
  const setting = page.getByRole('combobox', {name:'Graphics'});
  await expect(setting).toHaveValue('auto');
  await expect.poll(() => graphicsQuality(page)).toEqual({ratio:.75,shadow:512});
  await setting.selectOption('medium');
  await expect.poll(() => graphicsQuality(page)).toEqual({ratio:1.5,shadow:1024});
  await setting.selectOption('auto');
  await expect.poll(() => graphicsQuality(page)).toEqual({ratio:.75,shadow:512});
});
