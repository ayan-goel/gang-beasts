import { expect, test } from '@playwright/test';
import { decodeSnapshot } from '../src/net/snapshot';
import { approachOpponent, gripMatch } from './support/gripMatch';

if (process.platform === 'darwin') test.use({launchOptions:{args:['--use-gl=angle','--use-angle=metal']}});

test('losing the holding socket releases its grip and restores a neutral fighter', async ({page:host,browser}) => {
  let drop: (() => void) | undefined;
  const stopSteering = await approachOpponent(host, 0, disconnect => { drop = disconnect; });
  await gripMatch(host, browser, async (peer,start) => {
    const assignments: number[] = [];
    let inputAfterReturn: number[] | undefined;
    let cut = false, releasedAt: number | undefined, lastTick = 0, staleGrip = false;
    host.on('websocket', socket => {
      socket.on('framereceived', ({payload}) => {
        if (Buffer.isBuffer(payload) && payload[0] === 23) assignments.push(payload[2]!);
      });
      socket.on('framesent', ({payload}) => {
        if (Buffer.isBuffer(payload) && payload[0] === 1 && assignments.length === 2) {
          inputAfterReturn ??= [payload.readInt8(10),payload.readInt8(11),payload.readUInt16LE(12)];
        }
      });
    });
    peer.on('websocket', socket => socket.on('framereceived', ({payload}) => {
      if (!cut || !Buffer.isBuffer(payload) || payload[0] !== 2) return;
      const frame = decodeSnapshot(payload);
      lastTick = frame.tick;
      const held = frame.fighters[0]!.grips.some(Boolean);
      if (!held) releasedAt ??= frame.tick;
      if (held && releasedAt !== undefined) staleGrip = true;
    }));
    await start();
    await host.keyboard.down('KeyJ');
    try {
      for (const player of [host,peer]) {
        await expect(player.getByLabel('Mint grips')).toContainText('hand on Blue', {timeout:25_000});
      }
      stopSteering();
      expect(drop).toBeDefined();
      cut = true;
      drop!();
      await expect.poll(() => assignments, {timeout:15_000}).toEqual([0,0]);
      await expect.poll(() => inputAfterReturn).toEqual([0,0,0]);
      await expect.poll(() => releasedAt !== undefined && lastTick - releasedAt >= 60).toBe(true);
      expect(staleGrip).toBe(false);
      for (const player of [host,peer]) {
        await expect(player.getByLabel('Round number')).toHaveText('Round 1');
        await expect(player.getByLabel('Mint grips')).toHaveCount(0);
        await expect(player.getByLabel('Blue score')).toHaveText('0');
      }
    } finally { await host.keyboard.up('KeyJ'); }
  });
});
