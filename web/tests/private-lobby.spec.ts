import { expect, test } from '@playwright/test';

test('friends enter a private lobby from a shared link, reload and transfer the host role', async ({ page, browser }, info) => {
  await page.goto('/play/');
  await page.getByRole('link', { name: 'Private rooms' }).click();
  await page.getByLabel('Player name').fill('Coral');
  await page.getByLabel('Room capacity').selectOption('2');
  await page.getByRole('button', { name: 'Create room', exact: true }).click();
  await expect(page.getByRole('heading', { name: 'Room lobby' })).toBeVisible();
  const link = await page.getByLabel('Invite link').inputValue();
  expect(new URL(link).search).toMatch(/^\?code=[A-Z2-9]{6}$/);
  const guest = await browser.newContext();
  try {
    const blue = await guest.newPage();
    await blue.goto(link);
    await blue.getByLabel('Player name').fill('Blue');
    await blue.getByRole('button', { name: 'Join room', exact: true }).click();
    await expect(blue.getByRole('list', {name: 'Players'})).toContainText('Coral');
    await expect(page.getByRole('list', {name: 'Players'})).toContainText('Blue', {timeout: 12000});
    await page.reload();
    await expect(page.getByRole('list', {name: 'Players'})).toContainText('Blue');
    await page.screenshot({path:info.outputPath('private-lobby.png')});
    await page.getByRole('button',{name:'Leave room'}).click();
    await expect(page.getByLabel('Player name')).toBeVisible();
    await expect(blue.getByRole('list', {name:'Players'})).toContainText(/Blue.*Host/i, {timeout:12000});
    await expect(blue.getByRole('list', {name:'Players'})).not.toContainText('Coral');
  } finally {await guest.close();}
});

test('a narrow invite page handles an unknown code without losing the typed name', async ({page}) => {
  await page.setViewportSize({width:320,height:740});
  await page.goto('/rooms/?code=AAAAAA');
  await page.getByLabel('Player name').fill('Blue');
  await page.getByRole('button',{name:'Join room',exact:true}).click();
  await expect(page.getByRole('alert')).toContainText('Room code was not found');
  await expect(page.getByLabel('Player name')).toHaveValue('Blue');
  expect(await page.evaluate(()=>document.documentElement.scrollWidth)).toBe(320);
});

test('a seated player keeps a new invitation until leaving their current room', async ({page}) => {
  await page.goto('/rooms/');
  await page.getByLabel('Player name').fill('Coral');
  await page.getByRole('button', {name: 'Create room', exact: true}).click();
  await expect(page.getByLabel('Invite link')).toBeVisible();
  const current = (await page.getByLabel('Invite link').inputValue()).split('=')[1];
  const next = current === 'AAAAAA' ? 'BBBBBB' : 'AAAAAA';
  await page.goto('/rooms/?code=' + next);
  await expect(page.getByRole('status').filter({hasText: next})).toBeVisible();
  await page.reload();
  await page.getByRole('button', {name: 'Leave room'}).click();
  await expect(page.getByLabel('Room code')).toHaveValue(next);
  await expect(page).toHaveURL('/rooms/?code=' + next);
});
