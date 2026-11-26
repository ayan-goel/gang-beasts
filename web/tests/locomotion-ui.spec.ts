import { expect, test } from '@playwright/test';

test('focused movement walks, runs, releases on blur and pause, and resets cleanly', async ({page}) => {
  await page.goto('/playground/'); await expect(page.getByRole('status')).toHaveText('Simulation ready');
  await expect(page.locator('option[value="locomotion"]')).toHaveCount(1);
  await page.getByLabel('Test scene').selectOption('locomotion');
  await expect(page.locator('#movement-command')).toHaveText('Idle');
  await page.getByLabel('Inspect body').selectOption('9');
  await expect(page.locator('#motor-cap')).toHaveText('48.00 N·m');
  await page.getByRole('button', {name:'Resume',exact:true}).click();
  await page.getByRole('button', {name:'Focus movement controls'}).click();
  await page.keyboard.down('w');
  await expect(page.locator('#movement-command')).toHaveText('Walk');
  await expect(page.locator('#motor-cap')).toHaveText('150.00 N·m');
  await expect.poll(async()=>Number(await page.locator('#position-z').textContent())).toBeLessThan(-1);
  await page.keyboard.down('Shift');
  await expect(page.locator('#movement-command')).toHaveText('Run');
  await page.getByLabel('Gravity Y (m/s²)').focus();
  await expect(page.locator('#movement-command')).toHaveText('Idle');
  await page.keyboard.up('w'); await page.keyboard.up('Shift');
  await page.getByRole('button', {name:'Pause',exact:true}).click();
  await page.getByRole('button', {name:'Focus movement controls'}).click(); await page.keyboard.press('w');
  await expect(page.locator('#movement-command')).toHaveText('Idle');
  await page.getByRole('button', {name:'Reset fixture',exact:true}).click();
  await expect(page.locator('#position-z')).toHaveText('0.00');
  await expect(page.locator('#tick')).toHaveText('0');
  await page.getByLabel('Inspect body').selectOption('9');
  await expect(page.locator('#motor-cap')).toHaveText('48.00 N·m');
});

test('keyboard direction holds ignore unrelated releases and cannot restart from stale repeats', async ({page}) => {
  await page.goto('/playground/'); await expect(page.getByRole('status')).toHaveText('Simulation ready');
  await page.getByLabel('Test scene').selectOption('locomotion');
  await page.getByRole('button', {name:'Resume',exact:true}).click();
  const forward = page.getByRole('button', {name:'Move forward',exact:true});
  await forward.focus(); await page.keyboard.down('Space');
  await expect(page.locator('#movement-command')).toHaveText('Walk');
  await page.keyboard.press('Shift');
  await expect(page.locator('#movement-command')).toHaveText('Walk');
  await page.keyboard.up('Space');
  await expect(page.locator('#movement-command')).toHaveText('Idle');
  await page.keyboard.down('Enter');
  await expect(page.locator('#movement-command')).toHaveText('Walk');
  await page.evaluate(() => window.dispatchEvent(new Event('blur')));
  await expect(page.locator('#movement-command')).toHaveText('Idle');
  await forward.dispatchEvent('keydown', {key:'Enter',code:'Enter',repeat:true});
  await expect(page.locator('#movement-command')).toHaveText('Idle');
  await page.keyboard.up('Enter');
  await page.keyboard.down('Space');
  await expect(page.locator('#movement-command')).toHaveText('Walk');
  await page.getByLabel('Gravity Y (m/s²)').focus();
  await expect(page.locator('#movement-command')).toHaveText('Idle');
  await page.keyboard.up('Space');
});

test('pointer capture releases outside the button, visibility clears input, and scene changes remove handlers', async({page})=>{
  await page.goto('/playground/'); await expect(page.getByRole('status')).toHaveText('Simulation ready');
  await page.getByLabel('Test scene').selectOption('locomotion');
  await page.getByRole('button',{name:'Resume',exact:true}).click();
  const forward=page.getByRole('button',{name:'Move forward',exact:true});
  await forward.hover();await page.mouse.down();
  await expect(page.locator('#movement-command')).toHaveText('Walk');
  await page.mouse.move(10,10); await page.mouse.up();
  await expect(page.locator('#movement-command')).toHaveText('Idle');
  await page.getByRole('button',{name:'Focus movement controls'}).click();await page.keyboard.down('d');
  await expect(page.locator('#movement-command')).toHaveText('Walk');
  await page.evaluate(()=>{Object.defineProperty(document,'hidden',{configurable:true,value:true});document.dispatchEvent(new Event('visibilitychange'));});
  await expect(page.locator('#movement-command')).toHaveText('Idle'); await page.keyboard.up('d');
  await page.evaluate(()=>{delete (document as unknown as {hidden?:boolean}).hidden;});
  await page.getByLabel('Test scene').selectOption('standing');
  await expect(page.getByRole('button',{name:'Move forward',exact:true})).toHaveCount(0);
  await expect(page.locator('canvas')).toHaveCount(1);
  await page.getByLabel('Test scene').selectOption('locomotion');
  await expect(page.locator('#tick')).toHaveText('0');await expect(page.locator('#movement-command')).toHaveText('Idle');
});

test('narrow layouts reserve scene space above movement controls', async ({page}) => {
  await page.setViewportSize({width:320,height:700});
  await page.goto('/playground/'); await expect(page.getByRole('status')).toHaveText('Simulation ready');
  await page.getByLabel('Test scene').selectOption('locomotion');
  const layout = await page.evaluate(() => ({
    sceneBottom:document.querySelector('canvas')!.getBoundingClientRect().bottom,
    controlsTop:document.querySelector('.movement-controls')!.getBoundingClientRect().top,
    width:document.documentElement.scrollWidth,
  }));
  expect(layout.width).toBe(320);
  expect(layout.sceneBottom).toBeLessThanOrEqual(layout.controlsTop);
});
