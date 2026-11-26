import { expect, test, type Route } from '@playwright/test';

import { observeSkinDraws } from './support/skinDraws';

const fighterFile = /roof-crew[^/?]*\.glb(?:\?|$)/;

const failModel = (route: Route) => route.request().resourceType() === 'script' ? route.continue() : route.abort();

test('the local arena recovers from a missing fighter model and renders the match', async ({page}, info) => {
  const drawn = await observeSkinDraws(page, 2);
  await page.route(fighterFile, failModel);
  await page.goto('/play/');
  await expect(page.getByRole('alert')).toContainText('Could not load the fighters');
  await expect(page.getByRole('button',{name:'Start match'})).toBeDisabled();
  await page.unroute(fighterFile);
  await page.getByRole('button',{name:'Retry loading fighters'}).click({timeout:4000});
  await expect(page.getByRole('button',{name:'Start match'})).toBeEnabled();
  await page.getByRole('button',{name:'Start match'}).click();
  await expect(page.getByLabel('Round phase')).toHaveText('Playing', {timeout:15000});
  await expect.poll(drawn).toBe(2);
  await page.getByLabel('Skyline arena', {exact:true}).screenshot({path:info.outputPath('crew-match.png')});
});

test('online readiness waits for the fighter model and retries a failed download', async ({page}) => {
  await page.route(fighterFile, failModel);
  await page.goto('/rooms/');
  await page.getByLabel('Player name').fill('Mint');
  await page.getByRole('button',{name:'Create room',exact:true}).click();
  await page.getByRole('button',{name:'Ready up'}).click();
  await expect(page.getByRole('alert')).toContainText('Could not load Skyline');
  await expect(page.getByRole('listitem')).toContainText('Waiting');
  await page.unroute(fighterFile);
  await page.getByRole('button',{name:'Ready up'}).click();
  await expect(page.getByRole('button',{name:'Cancel ready'})).toBeVisible();
  await page.getByRole('button',{name:'Leave room'}).click();
});
