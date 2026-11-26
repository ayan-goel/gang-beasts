import {expect, test, type Browser, type Page} from '@playwright/test';
import {releaseRoom} from './releaseRoom';

export async function verifyOnlineSeries({page:host, browser}: {page:Page; browser:Browser}) {
  test.setTimeout(200_000);
  const profile = await browser.newContext();
  let invite: string | undefined;
  let completed = false;
  try {
    await host.goto('/rooms/');
    await host.getByLabel('Player name').fill('Mint');
    await host.getByLabel('Series target').selectOption('7');
    await host.getByRole('button',{name:'Create room',exact:true}).click();
    invite = await host.getByLabel('Invite link').inputValue();
    const guest = await profile.newPage();
    await guest.goto(invite);
    await guest.getByLabel('Player name').fill('Blue');
    await guest.getByRole('button',{name:'Join room',exact:true}).click();
    await expect(guest.getByText('First to 7 wins')).toBeVisible();
    await host.getByLabel('Series target').selectOption('3');
    await expect(guest.getByText('First to 3 wins')).toBeVisible();
    const players = [host,guest];
    for (const player of players) {
      await player.getByRole('button',{name:'Ready up'}).click();
      await expect(player.getByRole('button',{name:'Cancel ready'})).toBeVisible({timeout:12000});
    }
    await host.getByRole('button',{name:'Start match'}).click();
    const maps = new Set<string>();
    for (let round = 1; round <= 3; round++) {
      await expect(guest.getByLabel('Round number')).toHaveText('Round '+round,{timeout:15000});
      await expect(guest.getByLabel('Round phase')).toHaveText('Playing',{timeout:15000});
      const name = await guest.locator('.online-match').getAttribute('aria-label');
      expect(name).toBeTruthy();
      await expect(host.locator('.online-match')).toHaveAttribute('aria-label',name!);
      maps.add(name!);
      await guest.bringToFront();
      await guest.locator('.online-scene canvas').focus();
      await guest.keyboard.down('ShiftLeft');
      await guest.keyboard.down('KeyD');
      try {
        for (const player of players) {
          const standings = player.getByLabel('Series standings');
          await expect(standings).toBeVisible({timeout:30000});
          await expect(standings.getByLabel('Mint score')).toHaveText(String(round));
          await expect(standings.getByLabel('Blue score')).toHaveText('0');
        }
      } finally { await guest.keyboard.up('KeyD'); await guest.keyboard.up('ShiftLeft'); }
    }
    expect(maps.size).toBe(3);
    for (const player of players) {
      await expect(player.getByRole('status').filter({hasText:'Mint wins the series!'})).toBeVisible({timeout:10000});
    }
    for (const player of players) {
      await expect(player).toHaveURL(new URL('/',invite).href,{timeout:10000});
      await player.getByRole('link',{name:/Host a room/}).click();
      await expect(player.getByRole('button',{name:'Create room',exact:true})).toBeVisible();
    }
    completed = true;
    // A sleeping tab can reopen an old invite after missing the terminal socket event.
    await guest.goto(invite);
    await expect(guest).toHaveURL(new URL('/',invite).href);

  } finally {
    // Successful series retire every server-side seat, so only an interrupted test needs to leave.
    try { if (!completed) await releaseRoom(invite,[host.context(),profile]); }
    finally { await profile.close(); }
  }
}
