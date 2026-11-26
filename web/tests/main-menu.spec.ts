import { expect, test } from '@playwright/test';
import { releaseRoom } from './support/releaseRoom';

test('direct game URLs reach their screens with or without a trailing slash', async ({request}) => {
  for (const [path, title] of [
    ['/play', 'gang bbeasts · Local multiplayer'],
    ['/rooms', 'Private rooms · gang bbeasts'],
    ['/playground', 'gang bbeasts · Playground'],
  ]) {
    for (const suffix of ['', '/', '/index.html']) {
      const response = await request.get(path + suffix + '?action=join');
      expect(response.ok()).toBe(true);
      expect(new URL(response.url()).search).toBe('?action=join');
      expect(await response.text()).toContain('<title>' + title + '</title>');
    }
  }
});

test('the game menu works without JavaScript on desktop and narrow screens', async ({browser, browserName}, info) => {
  const context = await browser.newContext({javaScriptEnabled:false, viewport:{width:1440,height:900}});
  try {
    const page = await context.newPage();
    await page.goto('/');
    const poster = page.getByRole('img');
    expect(await poster.evaluate(element => (element as HTMLImageElement).naturalWidth)).toBeGreaterThan(0);
    // macOS WebKit uses Option-Tab for links when full keyboard access is off.
    const linkKey = process.platform === 'darwin' && browserName === 'webkit' ? 'Alt+Tab' : 'Tab';
    await page.keyboard.press(linkKey);
    await expect(page.getByRole('link', {name:'Host a room'})).toBeFocused();
    for (const width of [1440, 390, 320]) {
      await page.setViewportSize({width, height:width === 1440 ? 900 : 800});
      const fits = await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth);
      expect(fits).toBe(true);
      await expect(page.getByRole('link', {name:'Join a room'})).toBeInViewport();
      await expect(page.getByRole('link')).toHaveCount(2);
      await page.screenshot({path:info.outputPath('main-menu-' + width + '.png'), fullPage:true});
    }
  } finally { await context.close(); }
});

test('the title guide opens and closes by keyboard without loading a room or simulation', async ({page}, info) => {
  const heavy: string[] = [];
  page.on('request', request => {
    const pathname = new URL(request.url()).pathname;
    if (pathname.endsWith('.wasm') || pathname.startsWith('/api/')) heavy.push(pathname);
  });
  await page.goto('/');
  await expect(page.getByRole('heading', {level:1, name:'gang bbeasts'})).toBeVisible({timeout:3000});
  await expect(page.getByRole('link', {name:'Host a room'})).toBeVisible();
  await expect(page.getByRole('link', {name:'Join a room'})).toBeVisible();
  expect(heavy).toEqual([]);
  await expect(page.getByRole('link')).toHaveCount(2);
  await expect(page.getByRole('combobox')).toHaveCount(0);
  const guide = page.getByRole('dialog', {name:'How to play'});
  const opener = page.getByRole('button', {name:'How to play'});
  for (const width of [1280, 320]) {
    await page.setViewportSize({width,height:720});
    const bounds = await opener.boundingBox();
    expect(bounds!.x).toBeLessThan(30);
    expect(bounds!.y).toBeLessThan(30);
    await opener.focus();
    await opener.press('Enter');
    await expect(guide).toBeVisible();
    await expect(guide).toContainText('Punch & grab');
    await expect(guide).toContainText('3, 5, or 7 wins');
    await expect(guide.getByRole('button',{name:'Close guide'})).toBeFocused();
    expect(await guide.evaluate(el => el.scrollWidth <= el.clientWidth)).toBe(true);
    await page.screenshot({path:info.outputPath('guide-' + width + '.png')});
    if (width === 320) await guide.getByRole('button',{name:'Close guide'}).click();
    else await page.keyboard.press('Escape');
    await expect(guide).toBeHidden();
    await expect(opener).toBeFocused();
  }
  expect(heavy).toEqual([]);
});

test('friends can host and join by code from the main menu', async ({page, browser}, info) => {
  const guest = await browser.newContext({viewport:{width:390,height:800}});
  let invite = '';
  try {
    await page.goto('/');
    await page.getByRole('link', {name:'Host a room'}).click({timeout:3000});
    await expect(page.locator('header')).toHaveCount(0);
    await expect(page.getByLabel('Graphics', {exact:true})).toHaveCount(0);
    await page.getByLabel('Player name').fill('Mint');
    await page.getByRole('button', {name:'Create room', exact:true}).click();
    invite = await page.getByLabel('Invite link').inputValue();
    const friend = await guest.newPage();
    await friend.goto('/');
    await friend.getByRole('link', {name:'Join a room'}).click();
    await expect(friend.getByRole('button', {name:'Join', exact:true})).toHaveAttribute('aria-pressed', 'true');
    await friend.getByLabel('Room code').fill(new URL(invite).searchParams.get('code')!);
    await friend.getByLabel('Player name').fill('Blue');
    await friend.getByRole('button', {name:'Join room', exact:true}).click();
    await expect(friend.getByRole('heading', {name:'Room lobby'})).toBeVisible();
    await expect(page.getByText('Blue', {exact:true})).toBeVisible();
    for (const player of [friend, page]) await player.getByRole('button', {name:'Ready up'}).click();
    await page.getByRole('button', {name:'Start match'}).click();
    for (const [player, name] of [[page, 'Mint'], [friend, 'Blue']] as const) {
      await expect(player.getByRole('heading', {name:'Playing as ' + name})).toBeVisible();
      await expect(player.getByLabel('Round phase')).toHaveText('Playing', {timeout:15000});
      const canvas = player.locator('.online-scene canvas');
      await expect.poll(() => canvas.boundingBox()).toEqual({x:0,y:0,...player.viewportSize()});
      await expect(player.getByRole('button')).toHaveCount(0);
      await expect(player.getByRole('region',{name:'Scoreboard'})).toHaveCount(0);
      await expect(player.getByLabel('Series standings')).toHaveCount(0);
      expect(await player.evaluate(() => [[.05,.05],[.5,.5],[.95,.95]].every(([x,y]) =>
        document.elementFromPoint(innerWidth*x!,innerHeight*y!) instanceof HTMLCanvasElement))).toBe(true);
      await player.screenshot({path:info.outputPath('arena-' + name + '.png')});
    }
  } finally {
    try { await releaseRoom(invite,[page.context(),guest]); } finally { await guest.close(); }
  }
});
