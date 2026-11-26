import { expect, test, type BrowserContext, type Page } from '@playwright/test';
import { writeFile } from 'node:fs/promises';
import { verifyOnlineRematch } from './support/rematchScenario';
import { verifyDisconnectGrace } from './support/disconnectScenario';
import { releaseRoom } from './support/releaseRoom';
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
  } finally {
    try { await releaseRoom(link,[page.context(),profile]); }
    finally { await profile.close(); }
  }
});

test('the arena reports a stalled stream and resumes after ordered snapshots arrive', async ({page, browser}) => {
  let hold = false;
  let resume = () => {};
  let queued: Buffer[] = [];
  await page.routeWebSocket(url => url.pathname === '/ws', client => {
    const server = client.connectToServer();
    server.onMessage(message => {
      if (hold && Buffer.isBuffer(message) && message[0] === 2) queued.push(message);
      else client.send(message);
    });
    resume = () => {
      hold = false;
      for (const record of queued) client.send(record);
      queued = [];
    };
  });
  await page.goto('/rooms/');
  await page.getByLabel('Player name').fill('Coral');
  await page.getByRole('button',{name:'Create room', exact:true}).click();
  const link = await page.getByLabel('Invite link').inputValue();
  const profile = await browser.newContext();
  try {
    const guest = await profile.newPage();
    await guest.goto(link);
    await guest.getByLabel('Player name').fill('Blue');
    await guest.getByRole('button',{name:'Join room',exact:true}).click();
    for (const p of [page,guest]) await p.getByRole('button',{name:'Ready up'}).click();
    await page.getByRole('button',{name:'Start match'}).click();
    await expect(page.getByLabel('Round phase')).toHaveText('Playing', {timeout:15000});
    await expect(page.getByRole('heading',{name:'Playing as Coral'})).toBeVisible();
    hold = true;
    await expect(page.getByText('Waiting for the server…')).toBeVisible();
    await expect.poll(() => queued.length).toBeGreaterThan(8);
    await expect(guest.getByLabel('Round phase')).toHaveText('Playing');
    resume();
    await expect(page.getByText('Waiting for the server…')).toHaveCount(0);
    await expect(page.getByLabel('Round number')).toHaveText('Round 1');
    for (const p of [guest,page]) await p.getByRole('button',{name:'Leave room'}).click();
  } finally {
    resume();
    try { await releaseRoom(link,[page.context(),profile]); }
    finally { await profile.close(); }
  }
});

test('eight ready browsers share the rooftop and the eighth player can run off it', async ({browser}) => {
  const contexts: BrowserContext[] = [];
  const pages: Page[] = [];
  const eliminated = new Set<number>();
  let invite = '/rooms/';
  try {
    for (let slot = 0; slot < 8; slot++) {
      const context = await browser.newContext({viewport:{width:480, height:460}, deviceScaleFactor:0.65});
      contexts.push(context);
      const player = await context.newPage();
      pages.push(player);
      player.on('websocket', socket => socket.on('framereceived', ({payload}) => {
        if (!Buffer.isBuffer(payload) || payload[0] !== 2) return;
        const gameplay = 20 + payload.readUInt16LE(16) * 32;
        if (payload[gameplay+3] === 8 && payload[gameplay+24*8+1] === 3) eliminated.add(slot);
      }));
      await player.goto(invite);
      await player.getByLabel('Player name').fill('Fighter ' + (slot+1));
      await player.getByRole('button',{name:slot === 0 ? 'Create room' : 'Join room',exact:true}).click();
      if (slot === 0) invite = await player.getByLabel('Invite link').inputValue();
    }
    const host = pages[0]!;
    for (const player of pages.slice(0,7)) await player.getByRole('button',{name:'Ready up'}).click();
    await expect(host.getByRole('button',{name:'Start match'})).toBeDisabled();
    const eighth = pages[7]!;
    await eighth.getByRole('button',{name:'Ready up'}).click();
    await host.getByRole('button',{name:'Start match'}).click({timeout:10000});
    for (const [slot, player] of pages.entries()) {
      await expect(player.getByRole('heading',{name:'Playing as Fighter ' + (slot+1)})).toBeVisible();
      await expect(player.getByLabel('8 fighters on the Skyline rooftop')).toBeVisible();
      await expect(player.getByLabel('Fighter 8 score')).toHaveText('0');
    }
    await expect(eighth.getByLabel('Round phase')).toHaveText('Playing');
    await eighth.bringToFront();
    await eighth.locator('.online-scene canvas').focus();
    await eighth.keyboard.down('ShiftLeft');
    // From the eighth spawn the south edge is clear. Running east can hit the fixed rooftop vent.
    await eighth.keyboard.down('KeyS');
    try {
      await expect.poll(() => eliminated.size, {timeout:30000}).toBe(8);
    } finally {
      await eighth.keyboard.up('KeyS');
      await eighth.keyboard.up('ShiftLeft');
    }
    for (const player of pages) await expect(player.getByLabel('Fighter 8 eliminated')).toHaveText('Out');
    await expect(host.getByLabel('Round phase')).toHaveText('Playing');
  } finally {
    try {
      if (invite !== '/rooms/') await releaseRoom(invite,contexts);
    } finally {
      await Promise.all(contexts.map(context => context.close()));
    }
  }
});

test('an online seat reconnects before grace and spectates after expiry, then survivors return to the lobby', verifyDisconnectGrace);

test('the host and guest can play two rematches in the same room without refreshing', verifyOnlineRematch);
