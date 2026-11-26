import { test as base } from '@playwright/test';
export * from '@playwright/test';

export const test = base.extend({
  page: async ({page}, use) => {
    await page.addInitScript(() => {
      if (!/^\/play\/?$/.test(location.pathname)) return;
      const random = crypto.getRandomValues.bind(crypto);
      // Seed 4 opens the real shuffled series on Skyline. Recorded keyboard paths
      // in these tests expect its spawn positions, roof edges and hazard deadlines.
      Object.defineProperty(crypto,'getRandomValues',{value:(array:ArrayBufferView) => {
        if (array instanceof Uint32Array && array.length === 1) {
          array[0] = 4;
          return array;
        }
        return random(array);
      }});
    });
    await use(page);
  },
});
