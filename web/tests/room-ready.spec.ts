import { expect, test } from '@playwright/test';

test('two browsers see independent ready changes and a disconnected seat becomes unready', async ({page, browser}) => {
  await page.goto('/rooms/');
  await page.getByLabel('Player name').fill('Coral');
  await page.getByRole('button', {name: 'Create room', exact: true}).click();
  const invite = await page.getByLabel('Invite link').inputValue();
  const guest = await browser.newContext();
  try {
    const blue = await guest.newPage();
    await blue.goto(invite);
    await blue.getByLabel('Player name').fill('Blue');
    await blue.getByRole('button', {name: 'Join room', exact: true}).click();
    await expect(blue.getByRole('button', {name: 'Ready up'})).toBeEnabled();
    await blue.getByRole('button', {name: 'Ready up'}).click();
    const row = page.getByRole('listitem').filter({hasText: 'Blue'});
    await expect(row).toContainText('Ready', {timeout: 12000});
    await expect(page.getByRole('button', {name: 'Ready up'})).toBeEnabled();
    await blue.getByRole('button', {name: 'Cancel ready'}).click();
    await expect(row).toContainText('Waiting');
    await blue.getByRole('button', {name: 'Ready up'}).click();
    await expect(row).toContainText('Ready');
    await blue.close();
    await expect(row).toContainText('Waiting');
  } finally {
    await guest.close();
  }
});

test('reopening the same seat replaces its connection and the earlier tab can explicitly reconnect', async ({page, context}) => {
  await page.goto('/rooms/');
  await page.getByLabel('Player name').fill('Mint');
  await page.getByRole('button', {name: 'Create room', exact: true}).click();
  const invite = await page.getByLabel('Invite link').inputValue();
  await page.getByRole('button', {name: 'Ready up'}).click();
  await expect(page.getByRole('button', {name: 'Cancel ready'})).toBeVisible();
  const other = await context.newPage();
  await other.goto(invite);
  await expect(other.getByRole('button', {name: 'Ready up'})).toBeEnabled();
  await expect(page.getByRole('button', {name: 'Reconnect'})).toBeVisible();
  await page.getByRole('button', {name: 'Reconnect'}).click();
  await expect(page.getByRole('button', {name: 'Ready up'})).toBeEnabled();
  await expect(other.getByRole('button', {name: 'Reconnect'})).toBeVisible();
});
