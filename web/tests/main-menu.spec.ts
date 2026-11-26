import { expect, test, type Page } from '@playwright/test';
import { releaseRoom } from './support/releaseRoom';

function pageLoad(page: Page) {
  return page.evaluate(() =>
    [...performance.getEntriesByType('navigation'), ...performance.getEntriesByType('resource')]
      .filter(entry => entry.name.startsWith(location.origin + '/'))
      .map(entry => ({path:new URL(entry.name).pathname, bytes:(entry as PerformanceResourceTiming).encodedBodySize})));
}

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
      await expect(page.getByRole('link', {name:'Local match'})).toBeInViewport();
      await page.screenshot({path:info.outputPath('main-menu-' + width + '.png'), fullPage:true});
    }
  } finally { await context.close(); }
});

test('the main menu opens without a simulation or room session and reaches local play and practice', async ({page}, info) => {
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
  const menu = await pageLoad(page);
  await page.getByRole('link', {name:'Local match'}).click();
  await page.getByRole('button', {name:'Start match'}).click();
  await expect(page.getByLabel('Round phase')).toHaveText('Playing', {timeout:12000});
  const arena = await pageLoad(page);
  await info.attach('cold-load-to-first-arena', {body:JSON.stringify({menu, arena}), contentType:'application/json'});
  await page.getByRole('link', {name:'Main menu'}).click();
  await page.getByRole('link', {name:'Physics playground'}).click();
  await expect(page.getByRole('heading', {name:'Development playground'})).toBeVisible();
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
});

test('friends can host and join by code from the main menu', async ({page, browser}) => {
  const guest = await browser.newContext({viewport:{width:390,height:800}});
  let invite = '';
  try {
    await page.goto('/');
    await page.getByRole('link', {name:'Host a room'}).click({timeout:3000});
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
    }
  } finally {
    try { await releaseRoom(invite,[page.context(),guest]); } finally { await guest.close(); }
  }
});
