import { expect, test, type Locator } from '@playwright/test';
import { gripMatch } from './support/gripMatch';

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

test('a keyboard text choice carries from the main menu into a narrow local match and practice', async ({page}) => {
  await page.setViewportSize({width:320,height:800});
  await page.goto('/');
  const setting = page.getByRole('combobox', {name:'Text size', exact:true});
  const link = page.getByRole('link', {name:'Host a room'});
  const baseline = await fontSize(link);
  await setting.focus({timeout:4000});
  await setting.press('2');
  await setting.press('Tab');
  await expect.poll(() => fontSize(link)).toBeCloseTo(2 * baseline, 1);
  await fits(link);
  await page.getByRole('link', {name:'Local match'}).click();
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
  await expect.poll(() => fontSize(page.locator('html'))).toBeCloseTo(baseline*1.25, 1);
  await page.reload();
  await expect(setting).toHaveValue('125');
});

test('a private player can enlarge the live scoreboard independently of the other player', async ({page:host,browser}) => {
  await gripMatch(host,browser,async (guest,start) => {
    await start();
    await host.setViewportSize({width:320,height:800});
    const clock = host.getByLabel('Round time remaining');
    const baseline = await fontSize(clock);
    await host.getByRole('button',{name:'Match menu'}).click();
    const menu = host.getByRole('dialog');
    await menu.getByLabel('Text size').selectOption('200');
    await fits(menu.getByRole('button',{name:'Return to match'}));
    await menu.getByRole('button',{name:'Return to match'}).click();
    await expect(host.locator('canvas')).toBeFocused();
    await expect.poll(() => fontSize(clock)).toBeCloseTo(baseline*2,1);
    for (const label of ['Round time remaining','Mint score','Blue score']) await fits(host.getByLabel(label));
    await expect(guest.getByLabel('Text size')).toHaveValue('100');
    expect(await fontSize(guest.getByLabel('Round time remaining'))).toBeCloseTo(baseline,1);
    await expect(clock).not.toHaveText(await clock.innerText());
  });
});

test('menu and room form share all text sizes across open tabs', async ({page,context}) => {
  await page.goto('/');
  const lobby = await context.newPage();
  try {
    await lobby.setViewportSize({width:320,height:800});
    await lobby.goto('/rooms/');
    const field = lobby.getByLabel('Player name');
    const action = page.getByRole('link',{name:'Host a room'});
    const baseInput = await fontSize(field);
    const baseLink = await fontSize(action);
    for (const percent of [150,125,200,100]) {
      await page.getByRole('combobox',{name:'Text size'}).selectOption(String(percent));
      await expect(lobby.getByRole('combobox',{name:'Text size'})).toHaveValue(String(percent));
      await expect.poll(() => fontSize(field)).toBeCloseTo(baseInput * percent / 100, 1);
      expect(await fontSize(action)).toBeCloseTo(baseLink * percent / 100, 1);
      await fits(field);
      await fits(lobby.getByRole('button',{name:'Create room',exact:true}));
    }
  } finally { await lobby.close(); }
});
