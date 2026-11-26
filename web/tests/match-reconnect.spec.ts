import { expect, test } from '@playwright/test';
import { releaseRoom } from './support/releaseRoom';

test('a dropped match socket automatically recovers the same fighter with fresh controls', async ({ page, browser }) => {
  const commands: number[] = [];
  const epochs: bigint[] = [];
  const assignments: number[] = [];
  let drop = () => {};
  let restoredInput: number[] | undefined;
  let wasMoving = false;
  let movedAgain = false;
  let refuseNext = false;
  await page.routeWebSocket(url => url.pathname === '/ws', client => {
    if (refuseNext) {
      refuseNext = false;
      client.onMessage(() => client.close({code: 4012, reason: 'Temporary outage'}));
      return;
    }
    const server = client.connectToServer();
    client.onMessage(message => {
      if (Buffer.isBuffer(message)) {
        if (message[0] === 16 || message[0] === 32) commands.push(message[0]);
        if (message[0] === 24) epochs.push(message.readBigUInt64LE(2));
        if (message[0] === 1) {
          const axes = [message.readInt8(10), message.readInt8(11)];
          if (assignments.length === 1 && axes.some(axis => axis !== 0)) wasMoving = true;
          if (assignments.length === 2) {
            restoredInput ??= axes;
            if (axes.some(axis => axis !== 0)) movedAgain = true;
          }
        }
      }
      server.send(message);
    });
    server.onMessage(message => {
      if (Buffer.isBuffer(message) && message[0] === 23) assignments.push(message[2]!);
      client.send(message);
    });
    drop = () => {
      refuseNext = true;
      server.close({code: 1000});
      client.close({code: 4012, reason: 'Temporary outage'});
    };
  });
  const guestProfile = await browser.newContext();
  let invite: string | undefined;
  try {
    await page.goto('/rooms/');
    await page.getByLabel('Player name').fill('Mint');
    await page.getByRole('button', {name:'Create room', exact:true}).click();
    invite = await page.getByLabel('Invite link').inputValue();
    const guest = await guestProfile.newPage();
    await guest.goto(invite);
    await guest.getByLabel('Player name').fill('Gold');
    await guest.getByRole('button', {name:'Join room', exact:true}).click();
    for (const player of [guest, page]) await player.getByRole('button', {name:'Ready up'}).click();
    await page.getByRole('button', {name:'Start match'}).click();
    await expect(page.getByLabel('Round phase')).toHaveText('Playing', {timeout:15000});
    await page.locator('.online-scene canvas').focus();
    await page.keyboard.down('KeyW');
    await expect.poll(() => wasMoving).toBe(true);
    drop();
    await expect(page.getByRole('heading', {name:'Reconnecting to the match'})).toBeVisible();
    await expect(page.getByRole('heading', {name:'Playing as Mint'})).toBeVisible({timeout:15000});
    await expect.poll(() => restoredInput).toEqual([0,0]);
    await page.keyboard.up('KeyW');
    await page.keyboard.down('KeyD');
    await expect.poll(() => movedAgain).toBe(true);
    await page.keyboard.up('KeyD');
    expect(commands).toEqual([16,32]);
    expect(assignments).toEqual([0,0]);
    expect(epochs).toHaveLength(2);
    expect(epochs[1]).toBe(epochs[0]);
    await expect(guest.getByLabel('Round phase')).toHaveText('Playing');
    await expect(page.getByLabel('Mint score')).toHaveText('0');

    const successor = await page.context().newPage();
    await successor.goto(invite);
    await expect(successor.getByRole('heading', {name:'Playing as Mint'})).toBeVisible();
    await expect(page.getByRole('heading', {name:'Reconnecting to the match'})).toBeVisible();
    await expect(page.getByRole('button', {name:'Reconnect', exact:true})).toBeVisible({timeout:25000});
    await expect(successor.getByLabel('Round phase')).toHaveText('Playing');
    expect(commands.slice(2).length).toBeGreaterThan(0);
    expect(commands.slice(2).every(kind => kind === 32)).toBe(true);
    expect(assignments).toEqual([0,0]);
  } finally {
    try { await releaseRoom(invite, [page.context(), guestProfile]); }
    finally { await guestProfile.close(); }
  }
});
