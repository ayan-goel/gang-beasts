import { expect, test, type BrowserContext, type Page } from '@playwright/test';
import { writeFile } from 'node:fs/promises';
import { observeEighthFighter } from './support/onlineTrace';
import { releaseRoom } from './support/releaseRoom';
import { observeSkinDraws } from './support/skinDraws';

// Eight clients share this machine. Use its native GPU for the full-roster scenario on macOS.
if (process.platform === 'darwin') test.use({launchOptions:{args:['--use-gl=angle', '--use-angle=metal']}});

test('eight ready browsers share the rooftop and the eighth player can run off it', async ({browser}) => {
  const contexts: BrowserContext[] = [];
  const pages: Page[] = [];
  const traces: ReturnType<typeof observeEighthFighter>[] = [];
  let invite = '/rooms/';
  let drawn: (() => Promise<number>) | undefined;
  try {
    for (let slot = 0; slot < 8; slot++) {
      const context = await browser.newContext({viewport:{width:480, height:460}, deviceScaleFactor:0.65});
      contexts.push(context);
      const player = await context.newPage();
      pages.push(player);
      if (slot === 0) drawn = await observeSkinDraws(player, 8);
      traces.push(observeEighthFighter(player));
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
    await expect.poll(drawn!).toBe(8);
    await expect(eighth.getByLabel('Round phase')).toHaveText('Playing');
    await eighth.bringToFront();
    await eighth.locator('.online-scene canvas').focus();
    await eighth.keyboard.down('ShiftLeft');
    // From the eighth spawn the south edge is clear. Running east can hit the fixed rooftop vent.
    await eighth.keyboard.down('KeyS');
    try {
      await expect.poll(() => traces.map(trace => trace.eliminated), {timeout:30000}).toEqual(Array(8).fill(true));
    } finally {
      await eighth.keyboard.up('KeyS');
      await eighth.keyboard.up('ShiftLeft');
    }
    for (const player of pages) await expect(player.getByLabel('Fighter 8 eliminated')).toHaveText('Out');
    await expect(host.getByLabel('Round phase')).toHaveText('Playing');
  } catch (error) {
    const clients = await Promise.all(pages.map(async (p,slot) => ({
      fighter:slot+1, ...traces[slot],
      screen:await p.locator('main').innerText({timeout:2000}).catch(() => 'Page unavailable'),
    })));
    const path = test.info().outputPath('roster-delivery.json');
    await writeFile(path, JSON.stringify(clients));
    await test.info().attach('roster-delivery', {path,contentType:'application/json'});
    throw error;
  } finally {
    try {
      if (invite !== '/rooms/') await releaseRoom(invite,contexts);
    } finally {
      await Promise.all(contexts.map(context => context.close()));
    }
  }
});
