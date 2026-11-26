import { expect, test } from '@playwright/test';
import { releaseRoom } from './support/releaseRoom';

declare global { interface Window { lastArenaDraw?: WebGL2RenderingContext } }

test('ready draws the arena in advance, cancel releases it, and starting uses the prepared canvas', async ({page:host,browser}) => {
  const profile = await browser.newContext();
  let invite = '';
  try {
    const guest = await profile.newPage();
    await host.addInitScript(() => {
      const draw = WebGL2RenderingContext.prototype.drawElements;
      WebGL2RenderingContext.prototype.drawElements = function(...args) {
        window.lastArenaDraw = this;
        return draw.apply(this,args);
      };
    });
    await host.goto('/rooms/');
    await host.getByLabel('Player name').fill('Coral');
    await host.getByLabel('Fighter color').selectOption('5');
    await host.getByRole('button',{name:'Create room',exact:true}).click();
    invite = await host.getByLabel('Invite link').inputValue();
    const ready = host.getByRole('button',{name:'Ready up'});
    const cancel = host.getByRole('button',{name:'Cancel ready'});
    await ready.click();
    await expect(cancel).toBeVisible();
    expect(await host.evaluate(() => !!window.lastArenaDraw)).toBe(true);
    await expect(host.locator('canvas')).toHaveCount(0);
    const first = await host.evaluateHandle(() => window.lastArenaDraw!);
    await cancel.click();
    await expect.poll(() => first.evaluate(gl => gl.isContextLost())).toBe(true);
    await ready.click();
    await expect(cancel).toBeVisible();
    const prepared = await host.evaluateHandle(() => window.lastArenaDraw!);
    expect(await prepared.evaluate(gl => gl.isContextLost())).toBe(false);
    await guest.goto(invite);
    await guest.getByLabel('Player name').fill('Blue');
    await guest.getByRole('button',{name:'Join room',exact:true}).click();
    await guest.getByRole('button',{name:'Ready up'}).click();
    await host.getByRole('button',{name:'Start match'}).click();
    await expect(host.locator('canvas')).toBeVisible();
    expect(await prepared.evaluate(gl => gl.canvas === document.querySelector('canvas'))).toBe(true);
    await expect(host.getByLabel('Round phase')).toHaveText('Playing',{timeout:15000});
    await Promise.all([first.dispose(),prepared.dispose()]);
  } finally {
    try { await releaseRoom(invite,[host.context(),profile]); } finally { await profile.close(); }
  }
});

test('leaving during the first GPU draw releases the pending canvas', async ({page}) => {
  await page.addInitScript(() => {
    WebGL2RenderingContext.prototype.clientWaitSync = function() {
      window.lastArenaDraw = this;
      return this.TIMEOUT_EXPIRED;
    };
  });
  await page.goto('/rooms/');
  await page.getByLabel('Player name').fill('Mint');
  await page.getByRole('button',{name:'Create room',exact:true}).click();
  await page.getByRole('button',{name:'Ready up'}).click();
  await expect.poll(() => page.evaluate(() => !!window.lastArenaDraw)).toBe(true);
  await expect(page.getByRole('listitem')).toContainText('Waiting');
  const gl = await page.evaluateHandle(() => window.lastArenaDraw!);
  await page.getByRole('button',{name:'Leave room'}).click();
  await expect(page.getByRole('heading',{name:'Private rooms'})).toBeVisible();
  await expect.poll(() => gl.evaluate(context => context.isContextLost())).toBe(true);
  await gl.dispose();
});
