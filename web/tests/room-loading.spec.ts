import { expect, test } from '@playwright/test';

test('a failed fighter build download keeps the seat waiting and ready can retry', async ({page}) => {
  await page.goto('/rooms/');
  await page.getByLabel('Player name').fill('Coral');
  await page.getByRole('button', {name: 'Create room', exact: true}).click();
  await page.route(/\.wasm(?:\?|$)/, route => route.abort());
  await page.getByRole('button', {name: 'Ready up'}).click();
  await expect(page.getByRole('alert')).toContainText('Could not load Skyline');
  await expect(page.getByRole('listitem')).toContainText('Waiting');
  await page.unroute(/\.wasm(?:\?|$)/);
  await page.getByRole('button', {name: 'Ready up'}).click();
  await expect(page.getByRole('button', {name: 'Cancel ready'})).toBeVisible();
});

test('a new connection must load this engine and map before the server accepts readiness', async ({page}) => {
  await page.goto('/rooms/');
  await page.getByLabel('Player name').fill('Mint');
  await page.getByRole('button', {name: 'Create room', exact: true}).click();
  const invite = await page.getByLabel('Invite link').inputValue();
  const code = new URL(invite).searchParams.get('code')!;
  const replies = await page.evaluate(async code => {
    const socket = new WebSocket(new URL('/ws', location.origin.replace('http', 'ws')));
    socket.binaryType = 'arraybuffer';
    const exchange = (packet: Uint8Array<ArrayBuffer>) => new Promise<number[]>((resolve, reject) => {
      socket.onmessage = event => resolve([...new Uint8Array(event.data)]);
      socket.onclose = () => reject(new Error('Socket closed before response'));
      socket.send(packet);
    });
    await new Promise<void>((resolve, reject) => { socket.onopen = () => resolve(); socket.onerror = () => reject(); });
    try {
      const welcome = await exchange(Uint8Array.from([16, 1, ...new TextEncoder().encode(code)]));
      const premature = await exchange(Uint8Array.of(18, 1, 1));
      const incompatible = await exchange(Uint8Array.from([20, 1, ...new TextEncoder().encode('0'.repeat(128))]));
      const stillWaiting = await exchange(Uint8Array.of(18, 1, 1));
      return [welcome.slice(0, 2), premature, incompatible, stillWaiting];
    } finally { socket.close(); }
  }, code);
  expect(replies).toEqual([[17, 1], [21, 1, 2], [21, 1, 1], [21, 1, 2]]);
});

test('readiness waits for the online arena module to load', async ({page}) => {
  let requested = false;
  let release!: () => void;
  const gate = new Promise<void>(resolve => { release = resolve; });
  await page.route('**/online/OnlineMatch.tsx*', async route => {
    requested = true;
    await gate;
    await route.continue();
  });
  await page.goto('/rooms/');
  await page.getByLabel('Player name').fill('Mint');
  await page.getByRole('button',{name:'Create room',exact:true}).click();
  try {
    expect(requested).toBe(false);
    await page.getByRole('button',{name:'Ready up'}).click();
    await expect.poll(() => requested).toBe(true);
    await expect(page.getByRole('button',{name:'Loading Skyline…'})).toBeDisabled();
    await expect(page.getByRole('listitem')).toContainText('Waiting');
  } finally { release(); }
  await expect(page.getByRole('button',{name:'Cancel ready'})).toBeVisible();
});
