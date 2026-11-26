import { expect, test, type Route } from '@playwright/test';

for (const [asset, retry] of [['wasm','Retry physics download'], ['glb','Retry loading fighters']] as const) {
  test('local players can retry a stalled ' + asset + ' download without reloading', async ({page}) => {
    let first: Route | undefined;
    let downloads = 0;
    await page.route(new RegExp('\\.' + asset + '(?:\\?|$)'), async route => {
      if (route.request().resourceType() === 'script') return route.continue();
      if (++downloads === 1) first = route;
      else await route.continue();
    });
    let navigations = 0;
    page.on('framenavigated', frame => { if (frame === page.mainFrame()) navigations++; });
    try {
      await page.goto('/play/');
      await expect(page.getByRole('button',{name:retry})).toBeVisible({timeout:17000});
      await page.getByRole('button',{name:retry}).click();
      await page.getByRole('button',{name:'Start match'}).click({timeout:8000});
      await expect(page.getByLabel('Round phase')).toHaveText('Playing',{timeout:12000});
      expect(navigations).toBe(1);
      expect(downloads).toBe(2);
      await expect(page.getByRole('alert')).toHaveCount(0);
      await expect(page.locator('canvas')).toHaveCount(1);
    } finally { await first?.abort().catch(() => {}); }
  });
}

test('a slow physics download can finish on its own after offering a retry', async ({page}) => {
  let held: Route | undefined;
  let requests = 0;
  await page.route(/\.wasm(?:\?|$)/, async route => {
    if (route.request().resourceType() === 'script') return route.continue();
    if (++requests === 1) held = route;
    else await route.continue();
  });
  try {
    await page.goto('/playground/?scene=stairs');
    await expect(page.getByRole('button',{name:'Retry physics download'})).toBeVisible({timeout:17000});
    await held!.continue();
    held = undefined;
    await expect(page.getByRole('status')).toHaveText('Simulation ready');
    await expect(page.getByLabel('Test scene')).toHaveValue('stairs');
    expect(requests).toBe(1);
    await expect(page.getByRole('alert')).toHaveCount(0);
    await page.getByRole('button',{name:'Single step'}).click();
    await expect(page.locator('#tick')).toHaveText('1');
  } finally { await held?.abort().catch(() => {}); }
});

test('the physics engine starts with a generic binary content type', async ({page}) => {
  await page.route(/\.wasm(?:\?|$)/, async route => {
    if (route.request().resourceType() === 'script') return route.continue();
    const response = await route.fetch();
    await route.fulfill({response, contentType:'application/octet-stream'});
  });
  await page.goto('/play/');
  await page.getByRole('button',{name:'Start match'}).click();
  await expect(page.getByLabel('Round phase')).toHaveText('Playing',{timeout:12000});
});
