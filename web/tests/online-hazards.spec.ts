import {releaseRoom} from './support/releaseRoom';
import { expect, test } from '@playwright/test';

test('the online rooftop warns both players before each hazard and clears the warning at round end', async ({page:host, browser}) => {
  test.setTimeout(300_000);
  const profile = await browser.newContext();
  await host.goto('/rooms/');
  await host.getByLabel('Player name').fill('Mint');
  await host.getByRole('button', {name:'Create room', exact:true}).click();
  const invite = new URL(await host.getByLabel('Invite link').inputValue());
  try {
    const guest = await profile.newPage();
    await guest.goto(invite.href);
    await guest.getByLabel('Player name').fill('Blue');
    await guest.getByRole('button', {name:'Join room', exact:true}).click();
    for (const player of [guest,host]) await player.getByRole('button', {name:'Ready up'}).click();
    await host.getByRole('button', {name:'Start match'}).click();
    const notices = [host,guest].map(player => player.getByRole('status', {name:'Arena hazard'}));
    for (const notice of notices) await expect(notice).toHaveText('');
    for (const [message, deadline] of [
      ['West walkway failing', 110_000],
      ['East walkway failing', 45_000],
      ['High winds approaching', 45_000],
      ['Sudden death · Crosswind', 45_000],
    ] as const) {
      await Promise.all(notices.map(notice => expect(notice).toHaveText(message, {timeout:deadline})));
      if (message === 'East walkway failing') {
        await guest.reload();
        await expect(notices[1]!).toHaveText(message);
      }
    }
    await expect(host.getByLabel('Round phase')).toHaveText('Round results', {timeout:45_000});
    for (const notice of notices) await expect(notice).toHaveText('');
  } finally {
    try {
      await releaseRoom(invite.href,[host.context(),profile]);
    } finally { await profile.close(); }
  }
});
