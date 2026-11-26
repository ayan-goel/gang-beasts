import { expect, test, type Route } from '@playwright/test';

for (const asset of ['glb', 'wasm']) {
  test('a stalled ' + asset + ' download is cancelled and the same seat retries', async ({page}) => {
    const suffix = new RegExp('\\.' + asset + '(?:\\?|$)');
    let first: Route | undefined;
    let requests = 0;
    await page.route(suffix, async route => {
      if (route.request().resourceType() === 'script') return route.continue();
      if (++requests === 1) first = route;
      else await route.continue();
    });
    await page.addInitScript(extension => {
      const fetch = window.fetch.bind(window);
      window.fetch = (resource, options) => {
        const url = resource instanceof Request ? resource.url : String(resource);
        if (new URL(url, location.href).pathname.endsWith('.' + extension)) {
          const signal = options?.signal ?? (resource instanceof Request ? resource.signal : null);
          signal?.addEventListener('abort', () => document.documentElement.dataset.assetCancelled = 'yes', {once:true});
        }
        return fetch(resource, options);
      };
    },asset);
    await page.goto('/rooms/');
    await page.getByLabel('Player name').fill('Coral');
    await page.getByRole('button',{name:'Create room',exact:true}).click();
    const invite = await page.getByLabel('Invite link').inputValue();
    try {
      await page.getByRole('button',{name:'Ready up'}).click();
      await expect.poll(() => requests).toBe(1);
      await expect(page.getByRole('alert')).toContainText('too long',{timeout:16000});
      await expect(page.getByRole('listitem')).toContainText('Waiting');
      await expect(page.locator('html')).toHaveAttribute('data-asset-cancelled','yes');
      await page.getByRole('button',{name:'Ready up'}).click();
      await expect(page.getByRole('button',{name:'Cancel ready'})).toBeVisible({timeout:6500});
      expect(requests).toBe(2);
      expect(await page.getByLabel('Invite link').inputValue()).toBe(invite);
      await expect(page.getByRole('listitem')).toContainText('Coral (you)');
    } finally {
      await first?.abort().catch(() => {});
      await page.getByRole('button',{name:'Leave room'}).click();
    }
  });
}
