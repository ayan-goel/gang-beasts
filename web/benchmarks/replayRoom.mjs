import assert from 'node:assert/strict';
import { setTimeout as delay } from 'node:timers/promises';

const names = Array.from({length:8}, (_, i) => Buffer.from('Fighter ' + (i+1)));
const welcome = Buffer.from([17,1,0,255,0,0,1,0,0,0,8, ...names.flatMap((n,i) => [i,n.length,...n])]);
const assignment = Buffer.from([26,1,0,8, ...names.flatMap((n,i) => [i,i,n.length,...n])]);

export async function replayRoom(page, url, packets, identity, warmup, seconds) {
  let startPlayback;
  let contentIdentity;
  let lastInputTick = 0;
  const ready = new Promise(resolve => { startPlayback = resolve; });
  await page.route('**/api/session', route => route.fulfill({json:{active:true,room:'ABCDEF'}}));
  await page.routeWebSocket(socket => socket.pathname === '/ws', socket => {
    socket.onMessage(message => {
      if (message[0] === 16) socket.send(welcome);
      if (message[0] === 20) {
        contentIdentity = message.subarray(2).toString('utf8');
        socket.send(Buffer.from([21,1,0]));
      }
      if (message[0] === 18 && message[2] === 1) startPlayback(socket);
      if (message[0] === 1) lastInputTick = message.readUInt32LE(6);
    });
  });
  await page.goto(url + '/rooms/');
  await page.getByRole('button',{name:'Ready up'}).click();
  const socket = await Promise.race([ready, delay(10000, null, {ref:false})]);
  assert.ok(socket, 'Private match content never loaded');
  assert.equal(contentIdentity, identity, 'Browser content differs from the native capture');
  socket.send(assignment);
  // End every warmup at simulated second 30, so quick and full runs start with the same fight.
  const offset = (30 - warmup) * 30;
  const count = (warmup + seconds) * 30;
  assert.ok(offset >= 0 && offset + count <= packets.length);
  await page.evaluate(() => window.startProfile());
  const start = performance.now();
  let maxLate = 0;
  for (let i = 0; i < count; i++) {
    const due = start + i * 1000 / 30;
    await delay(Math.max(0, due - performance.now()));
    maxLate = Math.max(maxLate, performance.now() - due);
    socket.send(packets[offset+i]);
  }
  await delay(Math.max(0, start + (warmup+seconds)*1000 - performance.now()));
  assert.equal(await page.getByLabel(/^Fighter \d score$/).count(), 8);
  assert.equal(await page.getByLabel('Round phase').innerText(), 'Playing');
  const endTick = packets[offset+count-1].readUInt32LE(4);
  assert.ok(endTick - lastInputTick <= 6 && endTick >= lastInputTick, 'Client input must reference the final 100 ms of the replay');
  return {packets:count,endTick,lastInputTick,maxDispatchLagMs:Number(maxLate.toFixed(2))};
}
