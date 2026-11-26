import { breakawayPans, observeWarningAudio, warningSamples, windVoices } from './support/warningAudio';
import { arenaMeshes, authoredMeshes, observeArenaMeshes } from './support/arenaMeshes';
import {releaseRoom} from './support/releaseRoom';
import { expect, test } from '@playwright/test';

test('the online rooftop warns both players before each hazard and clears the warning at round end', async ({page:host, browser}) => {
  test.setTimeout(300_000);
  const profile = await browser.newContext();
  await Promise.all([observeWarningAudio(host), observeArenaMeshes(host)]);
  await host.goto('/rooms/');
  await host.getByLabel('Player name').fill('Mint');
  await host.getByRole('button', {name:'Create room', exact:true}).click();
  const invite = new URL(await host.getByLabel('Invite link').inputValue());
  try {
    const guest = await profile.newPage();
    await Promise.all([observeWarningAudio(guest), observeArenaMeshes(guest)]);
    await guest.goto(invite.href);
    await guest.getByLabel('Player name').fill('Blue');
    await guest.getByRole('button', {name:'Join room', exact:true}).click();
    await guest.getByRole('button',{name:'Mute game audio'}).click();
    for (const player of [guest,host]) await player.getByRole('button', {name:'Ready up'}).click();
    await host.getByRole('button', {name:'Start match'}).click();
    const players = [host,guest];
    const notices = players.map(player => player.getByRole('status', {name:'Arena hazard'}));
    for (const notice of notices) await expect(notice).toHaveText('');
    let hostCues = 0;
    for (const [message, deadline] of [
      ['West walkway failing', 110_000],
      ['East walkway failing', 45_000],
      ['High winds approaching', 45_000],
      ['Sudden death · Crosswind', 45_000],
    ] as const) {
      await Promise.all(notices.map(notice => expect(notice).toHaveText(message, {timeout:deadline})));
      await expect.poll(() => warningSamples(host)).toHaveLength(++hostCues);
      if (message !== 'Sudden death · Crosswind') {
        for (const player of players) expect(await windVoices(player)).toEqual({started:0,active:0});
      } else {
        await Promise.all(players.map(player => expect.poll(() => windVoices(player)).toEqual({started:1,active:1})));
        await Promise.all(players.map(player => expect.poll(async () =>
          (await arenaMeshes(player)).filter(mesh => !mesh.visible).map(mesh => mesh.name)
        ).toEqual(['panel-8', 'panel-9'])));
      }
      if (message === 'West walkway failing') {
        await Promise.all(players.map(player => expect.poll(async () =>
          (await arenaMeshes(player)).filter(mesh => mesh.visible && mesh.emissive !== 0).map(mesh => mesh.name)
        ).toEqual(['panel-8'])));
        expect(await warningSamples(guest)).toHaveLength(0);
        expect(await breakawayPans(guest)).toEqual([]);
        await guest.getByRole('button',{name:'Mute game audio'}).click();
      }
      if (message === 'East walkway failing') {
        await expect.poll(() => warningSamples(guest)).toHaveLength(1);
        for (const player of players) await expect.poll(() => breakawayPans(player)).toHaveLength(1);
        await guest.reload();
        await expect(notices[1]!).toHaveText(message);
        await guest.locator('canvas').focus();
        await guest.keyboard.press('Shift');
        expect(await warningSamples(guest)).toHaveLength(0);
        expect(await breakawayPans(guest)).toEqual([]);
      }
      if (message === 'High winds approaching') {
        await expect.poll(() => breakawayPans(host)).toHaveLength(2);
        await expect.poll(() => breakawayPans(guest)).toHaveLength(1);
      }
    }
    const [west,east] = await breakawayPans(host);
    expect(west! * east!).toBeLessThan(0);
    await expect.poll(() => warningSamples(guest)).toEqual([{wave:'sine',state:'running'}, {wave:'sine',state:'running'}]);
    await guest.reload();
    await expect(notices[1]!).toHaveText('Sudden death · Crosswind');
    await guest.locator('canvas').focus();
    await guest.keyboard.press('Shift');
    await expect.poll(() => windVoices(guest)).toEqual({started:1,active:1});
    expect(await warningSamples(guest)).toHaveLength(0);
    await Promise.all(players.map(async player => {
      await expect(player.getByLabel('Round phase')).toHaveText('Round results', {timeout:45_000});
      await expect.poll(() => windVoices(player)).toEqual({started:1,active:0});
      await expect(player.getByRole('status', {name:'Arena hazard'})).toHaveText('');
      const scores = await player.getByLabel(/^(Mint|Blue) score$/).allTextContents();
      expect(scores).toHaveLength(2);
      await expect(player.getByLabel('Round number')).toHaveText('Round 2', {timeout:10_000});
      await expect(player.getByLabel('Round phase')).toHaveText('Countdown');
      await expect.poll(() => arenaMeshes(player)).toEqual(authoredMeshes);
      await expect(player.getByRole('status', {name:'Arena hazard'})).toHaveText('');
      await expect(player.getByLabel(/^(Mint|Blue) score$/)).toHaveText(scores);
      await expect(player.getByLabel('Round phase')).toHaveText('Playing', {timeout:10_000});
      await expect(player.getByLabel(/^(Mint|Blue) eliminated$/)).toHaveCount(0);
    }));
  } finally {
    try {
      await releaseRoom(invite.href,[host.context(),profile]);
    } finally { await profile.close(); }
  }
});
