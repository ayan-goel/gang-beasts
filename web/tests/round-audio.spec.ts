import { expect, test } from '@playwright/test';
import { gripMatch } from './support/gripMatch';
import { observeWarningAudio, roundPitches } from './support/warningAudio';

test('the local countdown, ring-outs and match result sound once through a first-to-three', async ({page}) => {
  await observeWarningAudio(page);
  await page.goto('/play/');
  await page.getByRole('button',{name:'Start match'}).click();
  for (let round = 1; round <= 3; round++) {
    await expect(page.getByLabel('Round number')).toHaveText('Round '+round,{timeout:15000});
    await expect(page.getByLabel('Round phase')).toHaveText('Playing',{timeout:15000});
    await expect.poll(async () => (await roundPitches(page)).filter(hz => hz === 660).length).toBe(round);
    await page.keyboard.down('ShiftLeft');
    await page.keyboard.down('KeyA');
    try { await expect(page.getByLabel('Blue score')).toHaveText(String(round),{timeout:25000}); }
    finally { await page.keyboard.up('KeyA'); await page.keyboard.up('ShiftLeft'); }
  }
  await expect(page.getByRole('heading',{name:'Blue wins the match'})).toBeVisible({timeout:15000});
  await expect.poll(() => roundPitches(page)).toEqual([
    440,440,660,554, 440,440,440,660,554, 440,440,440,660,554,784,
  ]);
});

test('both private clients hear a scored round and reconnecting during results keeps it silent', async ({page:host,browser}) => {
  await gripMatch(host,browser,async (guest,start) => {
    await Promise.all([host,guest].map(observeWarningAudio));
    let disconnect = () => {};
    let connections = 0;
    await host.routeWebSocket(url => url.pathname === '/ws', client => {
      const server = client.connectToServer();
      connections++;
      disconnect = () => { server.close(); client.close({code:4012,reason:'Temporary outage'}); };
    });
    await start();
    for (const player of [host,guest]) {
      await expect.poll(() => roundPitches(player)).toEqual([440,440,660]);
    }
    await host.keyboard.down('ShiftLeft');
    await host.keyboard.down('KeyA');
    try { await expect(guest.getByLabel('Blue score')).toHaveText('1',{timeout:25000}); }
    finally { await host.keyboard.up('KeyA'); await host.keyboard.up('ShiftLeft'); }
    for (const player of [host,guest]) await expect.poll(() => roundPitches(player)).toEqual([440,440,660,554]);
    disconnect();
    await expect.poll(() => connections).toBe(2);
    await expect(host.getByLabel('Round phase')).toHaveText('Round results');
    await expect(host.getByLabel('Round number')).toHaveText('Round 2',{timeout:15000});
    await expect.poll(() => roundPitches(host)).toEqual([440,440,660,554,440]);
  });
});
