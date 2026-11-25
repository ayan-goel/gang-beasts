import { expect, test } from '@playwright/test';
import { execFileSync } from 'node:child_process';
import type { Snapshot } from '../src/net/snapshot';

test('Chromium reconstructs native Skyline grips, results, reset and retired arena bodies', async ({ page }) => {
  const lines = execFileSync('cargo', ['run', '--quiet', '--release', '--locked', '-p', 'brawler-server',
    '--example', 'skyline_frames'], { encoding: 'utf8' }).trim().split('\n');
  const frames = lines.map(line => Array.from(Buffer.from(line, 'hex')));
  await page.goto('/play/');
  const states = await page.evaluate(async packets => {
    const { decodeSnapshot } = await import(/* @vite-ignore */ String('/src/net/snapshot.ts'));
    return packets.map(packet => {
      const snap: Snapshot = decodeSnapshot(Uint8Array.from(packet));
      return { epoch: String(snap.round), phase: snap.phase, arena: snap.arenaTick,
        winner: snap.winner, scores: snap.fighters.map(f => f.score), count: snap.bodies.size,
        grips: snap.fighters[0]!.grips, life: snap.fighters[0]!.consciousness, ack: snap.applied,
        head: snap.bodies.get(1)!.position, west: snap.bodies.has(65536 + 8), east: snap.bodies.has(65536 + 9) };
    });
  }, frames);
  expect(states).toHaveLength(5);
  expect(states[0]).toMatchObject({ epoch: '1', phase: 0, count: 39, arena: 0, ack: null });
  expect(states[1]!.grips).toContain(65536 + 7);
  expect(states[1]!.ack).toEqual(states[1]!.arena - 1);
  expect(states[2]).toMatchObject({ epoch: '1', phase: 2, winner: 1, scores: [0, 1], life: 3, grips: [0, 0] });
  expect(states[3]).toMatchObject({ epoch: '2', phase: 0, arena: 0, scores: [0, 1], life: 0, ack: null,
    head: states[0]!.head, count: 39 });
  expect(states[4]).toMatchObject({ epoch: '2', phase: 1, arena: 9000, west: false, east: true });
});
