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

for (const [failure, action] of [['not_a_member','Rejoin room'], ['room_not_found','Find another room']] as const) {
  test('closed socket gives a recovery link for ' + failure, async ({page}) => {
    let disconnect = () => {};
    await page.routeWebSocket(url => url.pathname === '/ws', socket => {
      const server = socket.connectToServer();
      disconnect = () => { server.close(); socket.close(); };
    });
    await page.goto('/rooms/');
    await page.getByLabel('Player name').fill('Coral');
    await page.getByRole('button',{name:'Create room',exact:true}).click();
    await expect(page.getByLabel('Players',{exact:true}).getByRole('listitem')).toHaveCount(1);
    const invite = await page.getByLabel('Invite link').inputValue();
    await page.route('**/api/rooms/*/lobby', route => route.fulfill({
      status:failure === 'not_a_member' ? 403 : 404, json:{error:{code:failure}},
    }));
    disconnect();
    const link = page.getByRole('link',{name:action});
    await expect(link).toBeVisible();
    await expect(link).toHaveAttribute('href',failure === 'not_a_member' ? invite : '/rooms/');
    await expect(page.getByRole('button',{name:'Reconnect'})).toHaveCount(0);
  });
}
