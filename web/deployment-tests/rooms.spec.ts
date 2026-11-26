import { expect, test } from '@playwright/test';
import { releaseRoom } from '../tests/support/releaseRoom';

test('two first-party sessions play through direct sockets on a separate site', async ({ page, browser }, info) => {
  const backend = String(info.project.metadata.backend);
  const sockets: string[] = [];
  page.on('websocket', socket => sockets.push(new URL(socket.url()).host));
  const guest = await browser.newContext();
  let invite: string | undefined;
  try {
    await page.goto('/rooms/');
    await page.getByLabel('Player name').fill('Mint');
    await page.getByRole('button', {name: 'Create room', exact: true}).click();
    invite = await page.getByLabel('Invite link').inputValue();
    await expect(page.getByRole('button', {name: 'Ready up'})).toBeEnabled();
    await page.reload();
    await expect(page.getByLabel('Invite link')).toHaveValue(invite);
    const friend = await guest.newPage();
    await friend.goto(invite);
    await friend.getByLabel('Player name').fill('Gold');
    await friend.getByRole('button', {name: 'Join room', exact: true}).click();
    await friend.getByRole('button', {name: 'Ready up'}).click();
    await page.getByRole('button', {name: 'Ready up'}).click();
    await page.getByRole('button', {name: 'Start match'}).click();
    await expect(friend.getByLabel('Round phase')).toHaveText('Playing', {timeout: 15000});
    await expect(page.getByLabel('Round phase')).toHaveText('Playing');
    expect(sockets.length).toBeGreaterThanOrEqual(2);
    expect(new Set(sockets)).toEqual(new Set([new URL(backend).host]));
    for (const context of [page.context(), guest]) {
      expect(await context.cookies(backend)).toHaveLength(0);
      expect(await context.cookies(invite)).toEqual(expect.arrayContaining([
        expect.objectContaining({name: '__Host-skyline', httpOnly: true, sameSite: 'Strict'}),
      ]));
    }
    await friend.bringToFront();
    await friend.getByLabel('Online Skyline arena').locator('canvas').focus();
    await friend.keyboard.down('ShiftLeft');
    await friend.keyboard.down('KeyD');
    await expect(friend.getByLabel('Mint score')).toHaveText('1', {timeout: 20000});
    await expect(page.getByLabel('Mint score')).toHaveText('1');
  } finally {
    await releaseRoom(invite, [page.context(), guest]);
    await guest.close();
  }
});
