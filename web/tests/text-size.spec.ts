import { expect, test, type Locator } from '@playwright/test';

const fontSize = (element: Locator) => element.evaluate(node => parseFloat(getComputedStyle(node).fontSize));

async function fits(element: Locator) {
  await expect(element).toBeVisible();
  const bounds = await element.evaluate(node => {
    const box = node.getBoundingClientRect();
    return {left:box.left, right:box.right, viewport:innerWidth, page:document.documentElement.scrollWidth};
  });
  expect(bounds.left).toBeGreaterThanOrEqual(0);
  expect(bounds.right).toBeLessThanOrEqual(bounds.viewport + 1);
  expect(bounds.page).toBeLessThanOrEqual(bounds.viewport + 1);
}

test('the developer match and playground retain keyboard text size settings', async ({page}) => {
  await page.setViewportSize({width:320,height:800});
  await page.goto('/play/');
  const setting = page.getByRole('combobox', {name:'Text size', exact:true});
  const rootSize = await fontSize(page.locator('html'));
  await setting.focus();
  await setting.press('2');
  await setting.press('Tab');
  await expect(setting).toHaveValue('200');
  const start = page.getByRole('button', {name:'Start match'});
  await fits(start);
  await start.click();
  await expect(page.getByLabel('Round phase')).toHaveText('Playing', {timeout:15000});
  const board = page.getByRole('region', {name:'Scoreboard'});
  const clock = board.getByLabel('Round time remaining');
  const largeClock = await fontSize(clock);
  await page.keyboard.press('Tab');
  await expect(board).toBeFocused();
  for (const label of ['Coral score','Blue score','Round time remaining']) await fits(board.getByLabel(label));
  await fits(page.getByRole('button',{name:'Pause',exact:true}));
  const arena = (await page.locator('canvas').boundingBox())!;
  expect((await board.boundingBox())!.y).toBeGreaterThanOrEqual(arena.y + arena.height - 1);
  await page.keyboard.press('Escape');
  await expect(page.locator('canvas')).toBeFocused();
  await page.keyboard.press('Escape');
  const dialog = page.getByRole('dialog');
  await fits(dialog.getByLabel('Text size'));
  const clipped = await dialog.getByLabel('Camera motion').evaluate(node => {
    const select = node as HTMLSelectElement;
    const full = select.cloneNode(true) as HTMLSelectElement;
    full.style.cssText = 'position:fixed; visibility:hidden; width:max-content; max-width:none';
    select.parentElement!.append(full);
    const lost = full.offsetWidth - select.offsetWidth;
    full.remove();
    return lost;
  });
  expect(clipped).toBeLessThanOrEqual(1);
  const overflow = await dialog.evaluate(node => node.scrollWidth - node.clientWidth);
  expect(overflow).toBeLessThanOrEqual(1);
  await dialog.getByLabel('Text size').selectOption('100');
  await dialog.getByRole('button',{name:'Resume match'}).click();
  expect(await fontSize(clock)).toBeCloseTo(largeClock/2, 1);
  await setting.selectOption('125');
  await page.goto('/playground/');
  await expect(setting).toHaveValue('125');
  await expect.poll(() => fontSize(page.locator('html'))).toBeCloseTo(rootSize*1.25, 1);
  await page.reload();
  await expect(setting).toHaveValue('125');
});

test('public screens use default text size for returning players', async ({page}) => {
  await page.addInitScript(() => localStorage.setItem('skyline.display.v1.text-percent','200'));
  await page.setViewportSize({width:320,height:800});
  await page.goto('/');
  await fits(page.getByRole('link',{name:'Host a room'}));
  await expect(page.getByLabel('Text size')).toHaveCount(0);
  await page.getByRole('link',{name:'Host a room'}).click();
  await fits(page.getByLabel('Player name'));
  await expect(page.getByLabel('Text size')).toHaveCount(0);
  expect(await fontSize(page.locator('html'))).toBe(16);
});
