import { expect, test, type BrowserContext, type Page } from '@playwright/test';
import { writeFile } from 'node:fs/promises';
import { observeSnapshotTraffic } from './support/snapshotTraffic';
import { observeEighthFighter } from './support/onlineTrace';
import { releaseRoom } from './support/releaseRoom';
import { observeSkinDraws } from './support/skinDraws';

// Eight clients share this machine. Use its native GPU for the full-roster scenario on macOS.
if (process.platform === 'darwin') test.use({launchOptions:{args:['--use-gl=angle', '--use-angle=metal']}});

test('eight ready browsers share the rooftop and agree on the surviving fighter’s point', async ({browser}) => {
  test.setTimeout(240_000);
  const contexts: BrowserContext[] = [];
  const pages: Page[] = [];
  const traffic: ReturnType<typeof observeSnapshotTraffic>[] = [];
  const traces: ReturnType<typeof observeEighthFighter>[] = [];
  let invite = '/rooms/';
  let drawn: (() => Promise<number>) | undefined;
  try {
    for (let slot = 0; slot < 8; slot++) {
      const context = await browser.newContext({viewport:{width:480, height:460}, deviceScaleFactor:0.65});
      contexts.push(context);
      const player = await context.newPage();
      pages.push(player);
      traffic.push(observeSnapshotTraffic(player));
      if (slot === 7) {
        // Exercise a client that only advertises the original full snapshot format in the same match.
        await player.addInitScript(() => {
          const NativeSocket = window.WebSocket;
          window.WebSocket = class extends NativeSocket {
            constructor(url: string | URL, protocols?: string | string[]) {
              const target = new URL(url, location.href);
              if (target.pathname === '/ws') target.searchParams.delete('compact');
              super(target, protocols);
            }
          };
        });
      }
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
    await expect.poll(() => traffic.every(sample => sample.complete), {timeout:20000}).toBe(true);
    expect(traffic.map(sample => sample.compact)).toEqual([true,true,true,true,true,true,true,false]);
    await test.info().attach('snapshot-traffic', {
      body: JSON.stringify(traffic), contentType: 'application/json',
    });
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
    const exits = ['KeyD', 'KeyA', 'KeyD', 'KeyW', 'KeyD', 'KeyW'];
    for (const [offset, key] of exits.entries()) {
      const player = pages[offset + 1]!;
      await player.bringToFront();
      await player.locator('.online-scene canvas').focus();
      await player.keyboard.down('ShiftLeft');
      await player.keyboard.down(key);
      try {
        await expect(host.getByLabel('Fighter ' + (offset + 2) + ' eliminated')).toHaveText('Out', {timeout:25_000});
      } finally {
        await player.keyboard.up(key);
        await player.keyboard.up('ShiftLeft');
      }
    }
    await Promise.all(pages.map(async player => {
      const scores = player.getByLabel(/^Fighter \d score$/);
      const expected = ['1', '0', '0', '0', '0', '0', '0', '0'];
      await expect(player.getByRole('status').filter({hasText:'Fighter 1 wins the round'})).toBeVisible();
      await expect(scores).toHaveText(expected);
      await expect(player.getByLabel('Round number')).toHaveText('Round 2', {timeout:12_000});
      await expect(scores).toHaveText(expected);
      await expect(player.getByLabel(/^Fighter \d eliminated$/)).toHaveCount(0);
    }));
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
