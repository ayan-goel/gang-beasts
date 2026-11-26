import {releaseRoom} from './releaseRoom';
import {expect, type Browser, type Page} from '@playwright/test';

export async function verifyDisconnectGrace({page: host, browser}: {page: Page; browser: Browser}) {
  const profiles = await Promise.all([browser.newContext(), browser.newContext()]);
  const assignments: number[] = [];
  host.on('websocket', socket => socket.on('framereceived', ({payload}) => {
    if (Buffer.isBuffer(payload) && (payload[0] === 23 || payload[0] === 26)) assignments.push(payload[2]!);
  }));
  let invite: string | undefined;
  try {
    await host.goto('/rooms/');
    await host.getByLabel('Player name').fill('Mint');
    await host.getByRole('button',{name:'Create room', exact:true}).click();
    invite = await host.getByLabel('Invite link').inputValue();
    const peers = await Promise.all(profiles.map(context => context.newPage()));
    for (const [index, peer] of peers.entries()) {
      await peer.goto(invite);
      await peer.getByLabel('Player name').fill(['Gold','Blue'][index]!);
      await peer.getByRole('button',{name:'Join room', exact:true}).click();
      await peer.getByRole('button',{name:'Ready up'}).click();
    }
    await host.getByRole('button',{name:'Ready up'}).click();
    await host.getByRole('button',{name:'Start match'}).click();
    const [gold, blue] = peers as [Page, Page];
    await expect(host.getByLabel('Round phase')).toHaveText('Playing',{timeout:15000});
    expect(assignments).toEqual([0]);
    await host.goto('about:blank');
    await host.goto(invite);
    await expect.poll(() => assignments, {timeout:15_000}).toEqual([0,0]);
    await expect(host.getByRole('heading',{name:'Playing as Mint'})).toBeVisible({timeout:15_000});
    await expect(host.getByLabel('Round number')).toHaveText('Round 1');
    await expect(host.getByLabel('Mint eliminated')).toHaveCount(0);
    const disconnected = Date.now();
    await host.goto('about:blank');
    await expect(gold.getByLabel('Mint eliminated')).toHaveText('Out',{timeout:30000});
    expect(Date.now()-disconnected).toBeGreaterThanOrEqual(20000);
    await expect(blue.getByLabel('Mint eliminated')).toHaveText('Out');
    await expect(gold.getByLabel('Round phase')).toHaveText('Playing');
    await host.goto(invite);
    await host.getByLabel('Player name').fill('Mint again');
    await host.getByRole('button',{name:'Join room', exact:true}).click();
    await expect.poll(() => assignments, {timeout:15_000}).toEqual([0,0,255]);
    await expect(host.getByRole('heading',{name:'Spectating Skyline'})).toBeVisible({timeout:15_000});
    await expect(host.getByLabel('Mint eliminated')).toHaveText('Out');

    await blue.getByRole('button',{name:'Leave room'}).click();
    await Promise.all([host,gold].map(async player => {
      await expect(player.getByLabel('Gold score')).toHaveText('1');
      await expect(player.getByRole('heading',{name:'Room lobby'})).toBeVisible({timeout:10000});
      await expect(player.getByRole('button',{name:'Ready up'})).toBeVisible();
    }));
    await expect(gold.getByRole('button',{name:'Start match'})).toBeDisabled();
    await expect(host.getByRole('button',{name:'Start match'})).toHaveCount(0);
    for (const player of [host,gold]) await player.getByRole('button',{name:'Leave room'}).click();
  } finally {
    try { await releaseRoom(invite,[host.context(),...profiles]); }
    finally { await Promise.all(profiles.map(profile => profile.close())); }
  }
}
