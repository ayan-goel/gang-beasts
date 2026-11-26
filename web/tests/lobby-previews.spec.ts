import { expect, test } from '@playwright/test';
import { releaseRoom } from './support/releaseRoom';
import { readFileSync } from 'node:fs';

test('the lobby previews every playable arena before repeating, including across a shuffle boundary', async ({page}) => {
  const catalog = JSON.parse(readFileSync(new URL('../../assets/maps/tour.json',import.meta.url),'utf8')) as {tour:{name:string}}[];
  const expected = ['Skyline', ...catalog.map(map => map.tour.name)].sort();
  await page.clock.install();
  await page.goto('/rooms/');
  const gallery = page.getByRole('region', {name:'Arena previews'});
  const current = gallery.getByRole('figure').locator('strong');
  const seen = [(await current.textContent())!];
  for (let i = 0; i < expected.length; i++) {
    await expect.poll(async () => {
      await page.clock.fastForward(6000);
      return current.textContent();
    }).not.toBe(seen.at(-1) ?? '');
    seen.push((await current.textContent())!);
    await expect.poll(() => gallery.getByRole('figure').locator('img').evaluate(img =>
      (img as HTMLImageElement).naturalWidth)).toBeGreaterThan(0);
  }
  expect(seen.slice(0,-1).sort()).toEqual(expected);
  expect(seen.at(-1)).not.toBe(seen.at(-2));
});

test('a joiner sees reserved colors and can choose one after its owner leaves', async ({page:host,browser}) => {
  const guest = await browser.newContext();
  const rival = await browser.newContext();
  let invite = '';
  try {
    await host.goto('/rooms/');
    await host.getByLabel('Player name').fill('Mint');
    await host.getByLabel('Fighter color').selectOption({label:'Mint'});
    await host.getByRole('button',{name:'Create room',exact:true}).click();
    invite = await host.getByLabel('Invite link').inputValue();
    const joiner = await guest.newPage();
    await joiner.goto('/rooms/?action=join');
    const picker = joiner.getByLabel('Fighter color');
    await joiner.getByLabel('Room code').fill(new URL(invite).searchParams.get('code')!);
    await expect(picker.getByRole('option',{name:'Mint (taken)'})).toBeDisabled();
    await joiner.getByLabel('Player name').fill('Blue');
    await picker.selectOption({label:'Blue'});
    const other = await rival.newPage();
    await other.goto(invite);
    await other.getByLabel('Player name').fill('Rival');
    await other.getByLabel('Fighter color').selectOption({label:'Blue'});
    await other.getByRole('button',{name:'Join room',exact:true}).click();
    await expect(picker.getByRole('option',{name:'Blue (taken)'})).toBeDisabled({timeout:10000});
    await expect(joiner.getByRole('button',{name:'Join room',exact:true})).toBeDisabled();
    await other.getByRole('button',{name:'Leave room'}).click();
    await expect(picker.getByRole('option',{name:'Blue',exact:true})).toBeEnabled({timeout:10000});
    await joiner.getByRole('button',{name:'Join room',exact:true}).click();
    await expect(host.getByRole('list',{name:'Players'})).toContainText('Blue');
    for (const width of [1100,390]) {
      await host.setViewportSize({width,height:530});
      const panel = host.locator('.room-console');
      await expect.poll(() => panel.evaluate(el => el.scrollHeight - el.clientHeight)).toBeGreaterThan(0);
      const gallery = host.getByRole('region',{name:'Arena previews'});
      const before = await gallery.boundingBox();
      await host.getByRole('button',{name:'Leave room'}).scrollIntoViewIfNeeded();
      await expect(host.getByRole('button',{name:'Leave room'})).toBeInViewport();
      expect(await panel.evaluate(el => el.scrollTop)).toBeGreaterThan(0);
      expect(await gallery.boundingBox()).toEqual(before);
      expect(await host.evaluate(() => document.documentElement.scrollHeight)).toBe(530);
    }
  } finally {
    try { await releaseRoom(invite,[host.context(),guest,rival]); }
    finally { await Promise.all([guest.close(),rival.close()]); }
  }
});
