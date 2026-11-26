import { expect, test } from '@playwright/test';
import { measureLink } from './support/measureLink';
import { delayRelay } from './support/delayRelay';
import { gripMatch } from './support/gripMatch';

// Two software WebGL contexts on the same host can delay JavaScript delivery beyond the injected RTT.
if (process.platform === 'darwin') test.use({launchOptions:{args:['--use-gl=angle','--use-angle=metal']}});

test('private players see sustained high delay and recover on the same match connection', async ({page,browser,baseURL}, info) => {
  const relay = await delayRelay(baseURL!);
  const records: {configuredRttMs: number; perDirectionVariationMs: number; roundTripsMs: number[]}[] = [];
  let connections = 0;
  page.on('websocket', socket => { if (new URL(socket.url()).pathname === '/ws') connections++; });
  try {
    const samples = await measureLink(page, relay.port);
    await gripMatch(page,browser,async (guest,start) => {
      await start();
      const notice = page.getByText('High network delay. Controls may feel delayed.');
      for (const [rtt, variation] of [[20,0],[60,5],[100,0],[200,0],[20,0]] as const) {
        relay.profile(rtt, variation);
        const before = (await samples()).length;
        // Drop the first echo, which can straddle a profile switch, then measure five full probes.
        await expect.poll(async () => (await samples()).length - before,{timeout:9_000}).toBeGreaterThanOrEqual(6);
        const ms = (await samples()).slice(before + 1, before + 6);
        records.push({configuredRttMs:rtt, perDirectionVariationMs:variation, roundTripsMs:ms});
        const median = [...ms].sort((a,b) => a-b)[2]!;
        expect(median).toBeGreaterThanOrEqual(rtt - 2 * variation - 2);
        expect(median).toBeLessThan(rtt + 65);
        if (rtt === 200) await expect(notice).toBeVisible();
        else await expect(notice).toHaveCount(0);
        await expect(guest.getByText('High network delay. Controls may feel delayed.')).toHaveCount(0);
        await expect(page.getByLabel('Round phase')).toHaveText('Playing');
      }
      expect(connections).toBe(1);
    });
  } finally {
    await info.attach('transport-latency',{body:JSON.stringify(records), contentType:'application/json'});
    await relay.close();
  }
});

test('a match without the timing capability keeps its original protocol', async ({page,browser}) => {
  await page.addInitScript(() => {
    window.WebSocket = class extends WebSocket {
      constructor(url: string | URL, protocols?: string | string[]) {
        const path = new URL(url, location.href);
        path.searchParams.delete('timing');
        super(path, protocols);
      }
    };
  });
  const kinds = new Set<number>();
  page.on('websocket', socket => {
    const record = ({payload}: {payload: string | Buffer}) => {
      if (Buffer.isBuffer(payload)) kinds.add(payload[0]!);
    };
    socket.on('framesent', record);
    socket.on('framereceived', record);
  });
  await gripMatch(page,browser,async (_guest,start) => {
    await start();
    const clock = page.getByLabel('Round time remaining');
    await expect(clock).not.toHaveText(await clock.innerText());
    expect([1,2,23,24].every(kind => kinds.has(kind))).toBe(true);
    expect([35,36,37].some(kind => kinds.has(kind))).toBe(false);
  });
});
