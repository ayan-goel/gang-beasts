import { expect, test } from '@playwright/test';
import { writeFile } from 'node:fs/promises';
import { observeMatch } from './support/onlineTrace';

test('only the host can start and both ready browsers see the live rooftop and round', async ({page, browser}) => {
  const hostTrace = observeMatch(page);
  await page.goto('/rooms/');
  await page.getByLabel('Player name').fill('Mint');
  await page.getByRole('button', {name:'Create room',exact:true}).click();
  const link = await page.getByLabel('Invite link').inputValue();
  await expect(page.getByRole('button',{name:'Start match'})).toBeDisabled();
  const profile = await browser.newContext();
  try {
    const guest = await profile.newPage();
    const guestTrace = observeMatch(guest);
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
    for (const [player, key, winner, round] of [[guest, 'KeyD', 'Mint', 2], [page, 'KeyS', 'Gold', 3]] as const) {
      await player.bringToFront();
      await expect(player.getByLabel('Round number')).toHaveText('Round ' + (round - 1));
      await expect(player.getByLabel('Round phase')).toHaveText('Playing');
      await player.getByLabel('Online Skyline arena').locator('canvas').focus();
      await player.keyboard.down('ShiftLeft');
      await player.keyboard.down(key);
      try {
        await expect(page.getByLabel(winner + ' score')).toHaveText('1', {timeout:20000});
        await expect(guest.getByLabel(winner + ' score')).toHaveText('1');
      } catch (error) {
        const path = test.info().outputPath('physics-and-inputs.json');
        await writeFile(path, JSON.stringify({hostTrace, guestTrace}));
        await test.info().attach('native-poses-and-transmitted-inputs', {path, contentType:'application/json'});
        throw error;
      } finally {
        await player.keyboard.up(key);
        await player.keyboard.up('ShiftLeft');
      }
      await expect(player.getByLabel('Round number')).toHaveText('Round ' + round, {timeout:12000});
      await expect(player.getByLabel('Round phase')).toHaveText('Playing', {timeout:12000});
    }
    for (const player of [guest,page]) {
      await player.getByRole('button',{name:'Leave room'}).click();
      await expect(player.getByLabel('Player name')).toBeVisible();
    }
  } finally { await profile.close(); }
});
