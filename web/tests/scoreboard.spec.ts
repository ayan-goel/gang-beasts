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

test('a private scoreboard releases held input and stays live through a ring-out', async ({page:host,browser}) => {
  await gripMatch(host,browser,async (guest,start) => {
    let axis = 0, held = 0;
    host.on('websocket', socket => socket.on('framesent', ({payload}) => {
      if (Buffer.isBuffer(payload) && payload.length === 18 && payload[0] === 1) {
        axis = payload.readInt8(10);
        held = payload.readUInt16LE(12);
      }
    }));
    await start();
    await host.keyboard.down('KeyD');
    await host.keyboard.down('KeyJ');
    await expect.poll(() => axis !== 0 && (held & 4) !== 0).toBe(true);
    await host.keyboard.press('Tab');
    const board = host.getByRole('region', {name:'Scoreboard',exact:true});
    await expect(board).toBeFocused();
    await expect.poll(() => [axis,held]).toEqual([0,0]);
    await host.keyboard.up('KeyD');
    await host.keyboard.up('KeyJ');
    await guest.locator('canvas').focus();
    await guest.keyboard.down('ShiftLeft');
    await guest.keyboard.down('KeyD');
    try {
      await expect(board.getByLabel('Mint score')).toHaveText('1', {timeout:20000});
      await expect(guest.getByLabel('Mint score')).toHaveText('1');
    } finally {
      await guest.keyboard.up('KeyD');
      await guest.keyboard.up('ShiftLeft');
    }
    await expect(board).toBeFocused();
    await host.keyboard.press('Escape');
    await expect(host.locator('canvas')).toBeFocused();
  });
});
