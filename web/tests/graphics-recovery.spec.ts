import {expect, test, type Page} from '@playwright/test';
import {gripMatch} from './support/gripMatch';

async function loseGraphics(page: Page) {
  const extension = await page.locator('canvas').evaluateHandle((canvas: HTMLCanvasElement) => {
    const extension = canvas.getContext('webgl2')?.getExtension('WEBGL_lose_context');
    if (!extension) throw new Error('WebGL context-loss extension unavailable');
    extension.loseContext();
    return extension;
  });
  await expect(page.getByRole('alert')).toContainText('Graphics interrupted');
  return async () => {
    await extension.evaluate(control => control.restoreContext());
    await expect(page.getByText('Graphics interrupted', {exact:true})).toHaveCount(0);
    await extension.dispose();
  };
}

async function renderedColors(page: Page) {
  const image = await page.locator('canvas').screenshot();
  return page.evaluate(async png => {
    const bitmap = await createImageBitmap(await (await fetch('data:image/png;base64,' + png)).blob());
    const surface = new OffscreenCanvas(128,96);
    const context = surface.getContext('2d')!;
    context.drawImage(bitmap,0,0,128,96);
    bitmap.close();
    return new Set(new Uint32Array(context.getImageData(0,0,128,96).data.buffer)).size;
  },image.toString('base64'));
}

test('local graphics restore preserves the round clock and draws the arena again', async ({page}) => {
  await page.goto('/play/');
  await page.getByRole('button',{name:'Start match'}).click();
  await expect(page.getByLabel('Round phase')).toHaveText('Playing',{timeout:12000});
  const clock = page.locator('.match-round strong');
  await expect(clock).not.toHaveText(await clock.innerText());
  const restore = await loseGraphics(page);
  const frozen = await clock.innerText();
  await expect(page.getByRole('button',{name:'Reload and restart'})).toBeVisible();
  // Observe longer than a full scoreboard second while the GPU stays unavailable.
  await page.waitForTimeout(1400);
  expect(await clock.innerText()).toBe(frozen);
  await restore();
  await expect(page.locator('canvas')).toBeFocused();
  await expect(clock).not.toHaveText(frozen);
  await expect(page.getByLabel('Round number')).toHaveText('Round 1');
  await expect.poll(() => renderedColors(page)).toBeGreaterThan(100);
  await loseGraphics(page);
  await page.getByRole('button',{name:'Reload and restart'}).click();
  await expect(page.getByRole('button',{name:'Start match'})).toBeEnabled();
  await expect(page.getByLabel('Round phase')).toHaveText('Countdown');
});

test('private graphics loss releases held input while both seats stay in the round', async ({page:host,browser}) => {
  await gripMatch(host,browser,async (guest,start) => {
    let held = 0, axis = 0, connections = 0;
    host.on('websocket', socket => {
      connections++;
      socket.on('framesent', ({payload}) => {
        if (typeof payload !== 'string' && payload.length === 18 && payload[0] === 1) {
          held = payload.readUInt16LE(12); axis = payload.readInt8(10);
        }
      });
    });
    await start();
    const count = connections;
    await host.keyboard.down('KeyD');
    await host.keyboard.down('KeyJ');
    await expect.poll(() => axis !== 0 && (held & 4) !== 0).toBe(true);
    const restore = await loseGraphics(host);
    await expect.poll(() => [axis,held]).toEqual([0,0]);
    await expect(host.getByRole('button',{name:'Reload and reconnect'})).toBeVisible();
    for (const page of [host,guest]) {
      const clock = page.getByLabel('Round time remaining');
      await expect(clock).not.toHaveText(await clock.innerText());
    }
    await restore();
    expect(await renderedColors(host)).toBeGreaterThan(100);
    expect([axis,held]).toEqual([0,0]);
    await host.keyboard.up('KeyJ'); await host.keyboard.up('KeyD');
    await host.keyboard.down('KeyA');
    await expect.poll(() => axis).toBeLessThan(0);
    await host.keyboard.up('KeyA');
    expect(connections).toBe(count);
    await expect(host.getByRole('heading',{name:'Playing as Mint'})).toBeVisible();
  });
});

test('the playground keeps a paused fixture and disposes its recovery notice on scene change', async ({page}) => {
  await page.goto('/playground/');
  await expect(page.getByRole('status')).toHaveText('Simulation ready');
  await page.getByRole('button',{name:'Pause',exact:true}).click();
  const tick = page.locator('#tick');
  const pausedAt = await tick.innerText();
  const restore = await loseGraphics(page);
  await restore();
  await expect(page.getByRole('button',{name:'Resume',exact:true})).toBeVisible();
  await expect(tick).toHaveText(pausedAt);
  await page.getByRole('button',{name:'Single step'}).click();
  await expect(tick).toHaveText(String(Number(pausedAt)+1));
  await loseGraphics(page);
  await page.getByLabel('Test scene').selectOption('ragdoll');
  await expect(page.getByText('Graphics interrupted',{exact:true})).toHaveCount(0);
  await expect(page.locator('canvas')).toHaveCount(1);
  await expect.poll(() => renderedColors(page)).toBeGreaterThan(100);
});

test('reloading a private arena with a lost context reconnects the same fighter', async ({page:host,browser}) => {
  await gripMatch(host,browser,async (guest,start) => {
    await start();
    await loseGraphics(host);
    await host.getByRole('button',{name:'Reload and reconnect'}).click();
    await expect(host.getByRole('heading',{name:'Playing as Mint'})).toBeVisible({timeout:15000});
    await expect(host.getByLabel('Round phase')).toHaveText('Playing');
    await expect(guest.getByLabel('Round number')).toHaveText('Round 1');
    await expect.poll(() => renderedColors(host)).toBeGreaterThan(100);
  });
});
