import { expect, test, type Locator, type Page } from './support/skylineTest';
import { gripMatch } from './support/gripMatch';

declare global { interface Window { arenaLoops: {starts: number; stops: number} } }

async function observeLoops(page: Page) {
  await page.addInitScript(() => {
    const loops = window.arenaLoops = {starts:0, stops:0};
    window.AudioContext = class extends AudioContext {
      override createBufferSource() {
        const voice = super.createBufferSource();
        const start = voice.start.bind(voice), stop = voice.stop.bind(voice);
        voice.start = (...args) => { if (voice.loop) loops.starts++; start(...args); };
        voice.stop = (...args) => { if (voice.loop) loops.stops++; stop(...args); };
        return voice;
      }
    };
  });
}

const activeLoops = (page: Page) => page.evaluate(() => window.arenaLoops.starts - window.arenaLoops.stops);

async function setVolume(menu: Locator, name: string, value: number) {
  const slider = menu.getByRole('slider', {name: name + ' volume'});
  await slider.press('Home');
  for (let i = 0; i < Math.floor(value / 10); i++) await slider.press('PageUp');
  for (let i = 0; i < value % 10; i++) await slider.press('ArrowRight');
  await expect(slider).toHaveValue(String(value));
}

test('a local player can adjust the mix, pause music and restore the settings after reload', async ({page}) => {
  await observeLoops(page);
  await page.goto('/play/');
  await expect.poll(() => activeLoops(page)).toBe(0);
  await page.getByRole('button',{name:'Start match'}).click();
  await expect(page.getByLabel('Round phase')).toHaveText('Playing',{timeout:15000});
  await expect.poll(() => activeLoops(page)).toBe(1);
  await page.keyboard.press('Escape');
  const menu = page.getByRole('dialog');
  await expect.poll(() => activeLoops(page)).toBe(0);
  await setVolume(menu, 'Master', 70);
  await setVolume(menu, 'Music', 0);
  await setVolume(menu, 'Effects', 35);
  await menu.getByRole('button',{name:'Resume match'}).click();
  await expect(page.getByLabel('Round phase')).toHaveText('Playing');
  expect(await activeLoops(page)).toBe(0);
  await page.reload();
  await page.getByRole('button',{name:'Start match'}).click();
  await page.getByRole('button',{name:'Pause',exact:true}).click();
  for (const [name,value] of [['Master','70'],['Music','0'],['Effects','35']]) {
    await expect(menu.getByRole('slider',{name:name+' volume'})).toHaveValue(value!);
  }
});

test('room audio uses the default mix even for a returning muted player', async ({page:host,browser}) => {
  await host.addInitScript(() => localStorage.setItem('skyline.audio.v2', JSON.stringify({muted:true,music:0})));
  await gripMatch(host,browser,async (guest,start) => {
    await Promise.all([host,guest].map(observeLoops));
    await start();
    for (const page of [host,guest]) await expect.poll(() => activeLoops(page)).toBe(1);
    await host.keyboard.press('Escape');
    const menu = host.getByRole('dialog');
    await expect(menu.getByRole('button',{name:'Return to match'})).toBeVisible();
    await expect(menu.getByRole('slider')).toHaveCount(0);
    await expect(menu.getByRole('button',{name:'Mute game audio'})).toHaveCount(0);
  });
});
