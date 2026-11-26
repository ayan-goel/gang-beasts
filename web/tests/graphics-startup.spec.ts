import {expect, test, type Page} from '@playwright/test';
import {gripMatch} from './support/gripMatch';

async function blockGraphics(page: Page) {
  await page.addInitScript(() => {
    const getContext = HTMLCanvasElement.prototype.getContext;
    HTMLCanvasElement.prototype.getContext = function(this: HTMLCanvasElement, id:string, ...args:unknown[]) {
      if (id === 'webgl2' && !document.documentElement.hasAttribute('data-enable-gpu')) return null;
      return Reflect.apply(getContext,this,[id,...args]);
    } as typeof getContext;
  });
}

test('local play explains unavailable WebGL and retries when the GPU becomes available', async ({page}) => {
  await blockGraphics(page);
  await page.goto('/play/');
  await expect(page.getByRole('alert')).toContainText('WebGL 2');
  await expect(page.getByRole('button',{name:'Start match'})).toBeDisabled();
  await expect(page.locator('canvas')).toHaveCount(0);
  await page.evaluate(() => document.documentElement.setAttribute('data-enable-gpu',''));
  await page.getByRole('button',{name:'Retry graphics'}).click();
  await page.getByRole('button',{name:'Start match'}).click();
  await expect(page.getByLabel('Round phase')).toHaveText('Playing',{timeout:12000});
  await expect(page.locator('canvas')).toHaveCount(1);
});

test('a failed physics download can be retried without reloading the page', async ({page}) => {
  await page.addInitScript(() => { Object.defineProperty(window,'testBootId',{value:Math.random()}); });
  let requests = 0;
  await page.route(/\.wasm(?:\?|$)/, async route => {
    if (++requests === 1) await route.abort('failed');
    else await route.continue();
  });
  await page.goto('/play/');
  await expect(page.getByRole('alert')).toContainText('physics');
  const documentId = await page.evaluate(() => Reflect.get(window,'testBootId'));
  await page.getByRole('button',{name:'Retry physics download'}).click();
  await expect(page.getByRole('button',{name:'Start match'})).toBeEnabled();
  expect(requests).toBe(2);
  expect(await page.evaluate(() => Reflect.get(window,'testBootId'))).toBe(documentId);
});

test('a browser without WebAssembly gets a path back to the menu', async ({page}) => {
  await page.addInitScript(() => Object.defineProperty(window,'WebAssembly',{value:undefined}));
  await page.goto('/play/');
  await expect(page.getByRole('alert')).toContainText('browser with WebAssembly enabled');
  await expect(page.getByRole('button',{name:'Retry physics download'})).toHaveCount(0);
  await page.getByRole('link',{name:'Main menu'}).click();
  await expect(page).toHaveURL(/\/$/);
});

test('a private player retries graphics without leaving a live room', async ({page:host,browser}) => {
  await gripMatch(host,browser,async (guest,start) => {
    let connections = 0;
    guest.on('websocket',() => connections++);
    await blockGraphics(guest);
    await start();
    await expect(guest.getByRole('alert')).toContainText('WebGL 2');
    const original = connections;
    const clock = guest.getByLabel('Round time remaining');
    await expect(clock).not.toHaveText(await clock.innerText());
    await guest.evaluate(() => document.documentElement.setAttribute('data-enable-gpu',''));
    await guest.getByRole('button',{name:'Retry graphics'}).click();
    await expect(guest.locator('canvas')).toBeVisible();
    await expect(guest.getByLabel('Round phase')).toHaveText('Playing');
    await expect(guest.getByRole('heading',{name:'Playing as Blue'})).toBeVisible();
    expect(connections).toBe(original);
  });
});

test('a private lobby explains missing WebAssembly before readying the seat', async ({page}) => {
  await page.addInitScript(() => Object.defineProperty(window,'WebAssembly',{value:undefined}));
  await page.goto('/rooms/');
  await page.getByLabel('Player name').fill('Coral');
  await page.getByRole('button',{name:'Create room',exact:true}).click();
  try {
    await page.getByRole('button',{name:'Ready up'}).click();
    await expect(page.getByRole('alert')).toContainText('browser with WebAssembly enabled');
    await expect(page.getByRole('button',{name:'Ready up'})).toBeEnabled();
    await expect(page.getByRole('button',{name:'Start match'})).toBeDisabled();
  } finally { await page.getByRole('button',{name:'Leave room'}).click(); }
});
