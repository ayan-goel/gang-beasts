import { expect, test, type BrowserContext } from '@playwright/test';

test('eight friends join through the invite and a ninth sees the full-room message', async ({browser, page}) => {
  await page.goto('/rooms/');
  await page.getByLabel('Player name').fill('Host');
  await page.getByRole('button', {name: 'Create room', exact: true}).click();
  const invite = await page.getByLabel('Invite link').inputValue();
  const contexts: BrowserContext[] = [];
  try {
    for (let player = 2; player <= 9; player++) {
      const context = await browser.newContext();
      contexts.push(context);
      const guest = await context.newPage();
      await guest.goto(invite);
      await guest.getByLabel('Player name').fill('Player ' + player);
      await guest.getByRole('button', {name: 'Join room', exact: true}).click();
      if (player === 9) {
        await expect(guest.getByRole('alert')).toContainText('This room is full');
      } else {
        await expect(guest.getByRole('listitem')).toHaveCount(player);
        await expect(guest.getByRole('listitem').last()).toContainText('Player ' + player + ' (you)');
      }
    }
    await expect(page.getByRole('listitem')).toHaveCount(8, {timeout: 12000});
    await expect(page.getByRole('listitem').first()).toContainText('Host (you)');
    await expect(page.getByRole('listitem').last()).toContainText('Player 8');
  } finally {
    await Promise.all(contexts.map(c => c.close()));
  }
});
