import { expect, test } from '@playwright/test';
import { gripMatch } from './support/gripMatch';

test('Escape pauses both local fighters and resumes the same round', async ({page}) => {
  await page.goto('/play/');
  await page.getByRole('button', {name:'Start match'}).click();
  await expect(page.getByLabel('Round phase')).toHaveText('Playing', {timeout:15000});
  await page.keyboard.press('Escape');
  const menu = page.getByRole('dialog', {name:'Game paused'});
  await expect(menu).toBeVisible();
  const resume = menu.getByRole('button', {name:'Resume match'});
  await expect(resume).toBeFocused();
  const clock = page.locator('.match-round strong');
  const frozen = await clock.innerText();
  await menu.getByLabel('Graphics').selectOption('low');
  await page.waitForTimeout(1300);
  await expect(clock).toHaveText(frozen);
  await page.keyboard.press('Escape');
  await expect(menu).toHaveCount(0);
  await expect(page.locator('canvas')).toBeFocused();
  await expect(clock).not.toHaveText(frozen);
  await page.getByRole('button', {name:'Pause',exact:true}).click();
  await expect(menu.getByLabel('Graphics')).toHaveValue('low');
  await resume.click();
  await expect(page.locator('canvas')).toBeFocused();
});

test('the private menu releases held controls while both round clocks advance', async ({page:host,browser}) => {
  await gripMatch(host,browser,async (guest,start) => {
    let axis = 0, held = 0;
    host.on('websocket', socket => socket.on('framesent', ({payload}) => {
      if (typeof payload !== 'string' && payload.length === 18 && payload[0] === 1) {
        axis = payload.readInt8(10);
        held = payload.readUInt16LE(12);
      }
    }));
    await start();
    await host.keyboard.down('KeyD');
    await host.keyboard.down('KeyJ');
    await expect.poll(() => axis !== 0 && (held & 4) !== 0).toBe(true);
    await host.keyboard.press('Escape');
    const menu = host.getByRole('dialog', {name:'Match menu'});
    await expect(menu).toBeVisible();
    await expect.poll(() => [axis,held]).toEqual([0,0]);
    await expect(menu.getByRole('combobox')).toHaveCount(0);
    for (const player of [host,guest]) {
      const clock = player.getByLabel('Round time remaining');
      await expect(clock).not.toHaveText(await clock.innerText());
    }
    await host.keyboard.press('Escape');
    await expect(host.locator('canvas')).toBeFocused();
    expect([axis,held]).toEqual([0,0]);
    await host.keyboard.up('KeyD');
    await host.keyboard.up('KeyJ');
    await host.keyboard.down('KeyA');
    await expect.poll(() => axis).toBeLessThan(0);
    await host.keyboard.up('KeyA');
    await host.keyboard.press('Escape');
    await expect(menu.getByRole('button', {name:'Leave room'})).toBeVisible();
    await menu.getByRole('button', {name:'Return to match'}).click();
    await expect(host.locator('canvas')).toBeFocused();
  });
});
