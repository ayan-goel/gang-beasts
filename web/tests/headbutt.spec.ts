import {expect, test} from '@playwright/test';
import {decodeSnapshot} from '../src/net/snapshot';
import {releaseRoom} from './support/releaseRoom';

test('the shared keyboard spends stamina only for the fighter whose headbutt key was pressed', async ({page}) => {
  await page.goto('/playground/');
  await page.getByLabel('Test scene').selectOption('duel');
  await page.getByRole('button',{name:'Resume',exact:true}).click();
  for (const [key,name] of [['KeyQ','Coral'],['KeyM','Blue']] as const) {
    await page.locator('canvas').focus();
    await page.keyboard.press(key);
    await expect(page.getByLabel(name+' stamina')).toHaveText(/^9[4-9]$/);
  }
});

test('both online clients receive the native stamina change after one player taps Q', async ({page:host,browser}) => {
  const profile=await browser.newContext();
  let invite='';
  try {
    const guest=await profile.newPage();
    const observations=[false,false];
    for (const [i,p] of [host,guest].entries()) {
      p.on('websocket',socket=>socket.on('framereceived',({payload})=>{
        if (Buffer.isBuffer(payload) && payload[0]===2) {
          const [first,second]=decodeSnapshot(payload).fighters;
          if(first!.stamina===100 && second!.stamina<99) observations[i]=true;
        }
      }));
    }
    await host.goto('/rooms/');
    await host.getByLabel('Player name').fill('Mint');
    await host.getByRole('button',{name:'Create room',exact:true}).click();
    invite=await host.getByLabel('Invite link').inputValue();
    await guest.goto(invite);
    await guest.getByLabel('Player name').fill('Blue');
    await guest.getByRole('button',{name:'Join room',exact:true}).click();
    for (const p of [guest,host]) await p.getByRole('button',{name:'Ready up'}).click();
    await host.getByRole('button',{name:'Start match'}).click();
    await expect(guest.getByLabel('Round phase')).toHaveText('Playing',{timeout:15000});
    await guest.bringToFront();
    await guest.locator('.online-scene canvas').focus();
    await guest.keyboard.press('KeyQ');
    await expect.poll(()=>observations).toEqual([true,true]);
  } finally {
    try { if(invite) await releaseRoom(invite,[host.context(),profile]); }
    finally { await profile.close(); }
  }
});
