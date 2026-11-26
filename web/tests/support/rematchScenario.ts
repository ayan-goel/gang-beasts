import {releaseRoom} from './releaseRoom';
import {expect, test, type Browser, type Page} from '@playwright/test';

export async function verifyOnlineRematch({page:host, browser}: {page:Page; browser:Browser}) {
  test.setTimeout(200_000);
  const profile = await browser.newContext();
  let epoch = 0n, tick = 0;
  let activeHostSockets = 0;
  host.on('websocket', socket => {
    if (new URL(socket.url()).pathname !== '/ws') return;
    activeHostSockets++;
    socket.on('close', () => activeHostSockets--);
    socket.on('framereceived', ({payload}) => {
      if (!Buffer.isBuffer(payload) || payload[0] !== 2) return;
      const gameplay = 20 + 32 * payload.readUInt16LE(16);
      epoch = payload.readBigUInt64LE(gameplay + 8);
      tick = payload.readUInt32LE(4);
    });
  });
  let invite: string | undefined;
  try {
    await host.goto('/rooms/');
    await host.getByLabel('Player name').fill('Mint');
    await host.getByRole('button',{name:'Create room',exact:true}).click();
    invite = await host.getByLabel('Invite link').inputValue();
    const guest = await profile.newPage();
    let guestRematch = () => {};
    await guest.routeWebSocket(url => url.pathname === '/ws', socket => {
      const server = socket.connectToServer();
      guestRematch = () => server.send(Buffer.from([30,1]));
    });
    await guest.goto(invite);
    await guest.getByLabel('Player name').fill('Blue');
    await guest.getByRole('button',{name:'Join room',exact:true}).click();
    const pages = [host,guest];
    const origins = await Promise.all(pages.map(p => p.evaluate(() => performance.timeOrigin)));
    const start = async () => {
      for (const player of pages) await player.getByRole('button',{name:'Ready up'}).click();
      await host.getByRole('button',{name:'Start match'}).click();
    };
    await start();
    for (let replay=0; replay<2; replay++) {
      for (let round=1; round<=3; round++) {
        await expect(guest.getByLabel('Round number')).toHaveText('Round '+round,{timeout:15000});
        await expect(guest.getByLabel('Round phase')).toHaveText('Playing',{timeout:10000});
        await guest.bringToFront();
        await guest.locator('.online-scene canvas').focus();
        await guest.keyboard.down('ShiftLeft');
        await guest.keyboard.down('KeyD');
        try { await expect(host.getByLabel('Mint score')).toHaveText(String(round),{timeout:25000}); }
        finally { await guest.keyboard.up('KeyD'); await guest.keyboard.up('ShiftLeft'); }
      }
      for (const p of pages) await expect(p.getByLabel('Round phase')).toHaveText('Match results',{timeout:12000});
      const previousEpoch = epoch, lastTick = tick;
      await expect(guest.getByRole('button',{name:'Play again'})).toHaveCount(0);
      guestRematch();
      await expect.poll(() => tick).toBeGreaterThan(lastTick+8);
      await expect(host.getByLabel('Round phase')).toHaveText('Match results');
      const oldContexts = await Promise.all(pages.map(p => p.locator('.online-scene canvas').evaluateHandle(
        canvas => (canvas as HTMLCanvasElement).getContext('webgl2')!,
      )));
      await host.getByRole('button',{name:'Play again'}).click();
      for (const p of pages) {
        await expect(p.getByRole('heading',{name:'Room lobby'})).toBeVisible();
        expect(await p.getByLabel('Invite link').inputValue()).toBe(invite);
      }
      for (const gl of oldContexts) {
        await expect.poll(() => gl.evaluate(context => context.isContextLost())).toBe(true);
        await gl.dispose();
      }
      await expect(host.getByRole('button',{name:'Start match'})).toBeDisabled();
      await start();
      for (const [i,p] of pages.entries()) {
        await expect(p.getByLabel('Round number')).toHaveText('Round 1');
        await expect(p.getByLabel('Round phase')).toHaveText('Playing',{timeout:10000});
        await expect(p.getByLabel('Mint score')).toHaveText('0');
        await expect(p.getByLabel('Blue score')).toHaveText('0');
        expect(await p.evaluate(() => performance.timeOrigin)).toBe(origins[i]);
      }
      expect(epoch).toBeGreaterThan(previousEpoch);
      await expect.poll(() => activeHostSockets).toBe(1);
    }
    for (const p of [guest,host]) await p.getByRole('button',{name:'Leave room'}).click();
  } finally {
    try { await releaseRoom(invite,[host.context(),profile]); }
    finally { await profile.close(); }
  }
}
