import { expect, test } from '@playwright/test';
import { gripMatch } from './support/gripMatch';

test('Tab reads the local scoreboard during play and still permits ordinary keyboard navigation', async ({page}) => {
  await page.goto('/play/');
  await page.getByRole('button',{name:'Start match'}).click();
  await expect(page.getByLabel('Round phase')).toHaveText('Playing',{timeout:15000});
  const board = page.getByRole('region', {name:'Scoreboard',exact:true});
  await page.keyboard.press('Tab');
  await expect(board).toBeFocused();
  await expect(board.getByLabel('Coral score')).toHaveText('0');
  await expect(board.getByLabel('Blue score')).toHaveText('0');
  const clock = board.getByLabel('Round time remaining');
  await expect(clock).not.toHaveText(await clock.innerText());
  await page.keyboard.press('Escape');
  await expect(page.locator('canvas')).toBeFocused();
  await expect(page.getByRole('dialog')).toHaveCount(0);
  await page.keyboard.press('Tab');
  await page.keyboard.press('Shift+Tab');
  await expect(page.locator('canvas')).toBeFocused();
  await page.keyboard.press('Tab');
  await page.keyboard.press('Tab');
  await expect(board).not.toBeFocused();
  await expect(page.locator('canvas')).not.toBeFocused();
});

test('private players see the score together at the round break', async ({page:host,browser}) => {
  await gripMatch(host,browser,async (guest,start) => {
    await start();
    for (const page of [host,guest]) {
      await expect(page.getByLabel('Series standings')).toHaveCount(0);
      await expect(page.getByRole('region',{name:'Scoreboard'})).toHaveCount(0);
    }
    await guest.locator('canvas').focus();
    await guest.keyboard.down('ShiftLeft');
    await guest.keyboard.down('KeyD');
    try {
      for (const page of [host,guest]) {
        const standings = page.getByLabel('Series standings');
        await expect(standings).toBeVisible({timeout:25000});
        await expect(standings.getByLabel('Mint score')).toHaveText('1');
        await expect(standings.getByLabel('Blue score')).toHaveText('0');
      }
    } finally { await guest.keyboard.up('KeyD'); await guest.keyboard.up('ShiftLeft'); }
  });
});
