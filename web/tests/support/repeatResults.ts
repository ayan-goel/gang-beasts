import type { Page } from '@playwright/test';
import { decodeSnapshot } from '../../src/net/snapshot';

export async function repeatResults(page: Page) {
  const deliveries = { duplicate: 0, previousRound: 0 };
  await page.routeWebSocket(url => url.pathname === '/ws', client => {
    const server = client.connectToServer();
    let result: { round: bigint; bytes: Buffer } | undefined;
    server.onMessage(message => {
      client.send(message);
      if (!Buffer.isBuffer(message) || message[0] !== 2) return;
      const frame = decodeSnapshot(message);
      if (result && frame.round > result.round) {
        // Re-deliver the old verdict after the first new-round snapshot has reached this client.
        client.send(result.bytes);
        result = undefined;
        deliveries.previousRound++;
      }
      if (frame.phase === 2 && !result) {
        result = { round: frame.round, bytes: message };
        client.send(message);
        deliveries.duplicate++;
      }
    });
  });
  return deliveries;
}
