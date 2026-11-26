import {expect, test} from '@playwright/test';
import {releaseRoom} from './support/releaseRoom';

test('both local fighters follow the rotating view while holding forward', async ({page}) => {
  await page.goto('/play/');
  const start = page.getByRole('button',{name:'Start match'});
  await expect(start).toBeEnabled();
  await page.evaluate(async () => {
    const {LocalDuel} = await import(/* @vite-ignore */ String('/src/generated/wasm/brawler_wasm.js'));
    const move = LocalDuel.prototype.set_input;
    LocalDuel.prototype.set_input = function(id: number, x: number, z: number, run: boolean) {
      document.documentElement.dataset['direction' + id] = x.toFixed(2) + ',' + z.toFixed(2);
      return move.call(this,id,x,z,run);
    };
  });
  await start.click();
  await expect(page.getByLabel('Round phase')).toHaveText('Playing',{timeout:12000});
  await page.keyboard.down('KeyW');
  await page.keyboard.down('ArrowUp');
  const directions = () => page.evaluate(() => [0,1].map(id => document.documentElement.dataset['direction'+id]));
  await expect.poll(directions).toEqual(['-0.18,-0.98','-0.18,-0.98']);
  await page.keyboard.press('KeyX');
  await expect.poll(directions).toEqual(['-0.82,-0.57','-0.82,-0.57']);
  await page.keyboard.up('KeyW');
  await page.keyboard.up('ArrowUp');
  await expect.poll(directions).toEqual(['0.00,0.00','0.00,0.00']);
});

test('turning one online camera changes only that client’s held movement packets', async ({page:host,browser}) => {
  const profile = await browser.newContext();
  let invite: string | undefined;
  try {
    const guest = await profile.newPage();
    const players = [host,guest];
    const directions = [[0,0],[0,0]];
    players.forEach((page,i) => page.on('websocket', socket => socket.on('framesent', ({payload}) => {
      if (Buffer.isBuffer(payload) && payload[0] === 1 && payload.length === 18) {
        directions[i] = [payload.readInt8(10), payload.readInt8(11)];
      }
    })));
    await host.goto('/rooms/');
    await host.getByLabel('Player name').fill('Mint');
    await host.getByRole('button',{name:'Create room',exact:true}).click();
    invite = await host.getByLabel('Invite link').inputValue();
    await guest.goto(invite);
    await guest.getByLabel('Player name').fill('Blue');
    await guest.getByRole('button',{name:'Join room',exact:true}).click();
    for (const p of players) await p.getByRole('button',{name:'Ready up'}).click();
    await host.getByRole('button',{name:'Start match'}).click();
    for (const p of players) await expect(p.getByLabel('Round phase')).toHaveText('Playing',{timeout:15000});
    await host.bringToFront();
    await host.locator('.online-scene canvas').focus();
    await host.keyboard.down('KeyW');
    await expect.poll(() => directions[0]).toEqual([-22,-125]);
    await host.keyboard.press('KeyX');
    await expect.poll(() => directions[0]).toEqual([-104,-72]);
    await host.keyboard.up('KeyW');
    await guest.bringToFront();
    await guest.locator('.online-scene canvas').focus();
    await guest.keyboard.down('KeyW');
    await expect.poll(() => directions[1]).toEqual([-22,-125]);
    await guest.keyboard.up('KeyW');
  } finally {
    try { await releaseRoom(invite,[host.context(),profile]); }
    finally { await profile.close(); }
  }
});
