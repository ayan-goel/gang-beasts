import { expect, test } from '@playwright/test';

test('only the host can start and both ready browsers see the live rooftop and round', async ({page, browser}) => {
  await page.goto('/rooms/');
  await page.getByLabel('Player name').fill('Mint');
  await page.getByRole('button', {name:'Create room',exact:true}).click();
  const link = await page.getByLabel('Invite link').inputValue();
  await expect(page.getByRole('button',{name:'Start match'})).toBeDisabled();
  const profile = await browser.newContext();
  try {
    const guest = await profile.newPage();
    await guest.goto(link);
    await guest.getByLabel('Player name').fill('Gold');
    await guest.getByRole('button',{name:'Join room',exact:true}).click();
    await guest.getByRole('button',{name:'Ready up'}).click();
    await expect(guest.getByRole('button',{name:'Start match'})).toHaveCount(0);
    await expect(page.getByRole('button',{name:'Start match'})).toBeDisabled();
    await page.getByRole('button',{name:'Ready up'}).click();
    await page.getByRole('button',{name:'Start match'}).click();
    for (const player of [page,guest]) {
      await expect(player.getByLabel('Online Skyline arena').locator('canvas')).toBeVisible();
      await expect(player.getByLabel('Round number')).toHaveText('Round 1');
      await expect(player.getByLabel('Round phase')).toHaveText('Playing', {timeout:12000});
      await expect(player.getByLabel('Mint score')).toHaveText('0');
      await expect(player.getByLabel('Gold score')).toHaveText('0');
    }
    await guest.reload();
    await guest.getByRole('button',{name:'Ready up'}).click();
    await expect(guest.getByLabel('Online Skyline arena').locator('canvas')).toBeVisible();
    await expect(guest.getByLabel('Round phase')).toHaveText('Playing');
  } finally { await profile.close(); }
});
