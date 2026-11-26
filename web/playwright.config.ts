import { defineConfig } from '@playwright/test';

const tls = Boolean(process.env.BRAWLER_TLS_CERT && process.env.BRAWLER_TLS_KEY);
const site = (tls ? 'https' : 'http') + '://127.0.0.1:5180';
const server = 'http://127.0.0.1:5181';
const releaseMeasurement = [
  'BRAWLER_CHECK_REGIONAL_JUMP', 'BRAWLER_CHECK_REGIONAL_PUNCH', 'BRAWLER_CHECK_REGIONAL_HEADBUTT',
].some(flag => process.env[flag] === '1');
const webCommand = releaseMeasurement ? 'npm run build && npm run preview' : 'npm run dev';

export default defineConfig({
  testDir: './tests',
  // The eight-player scenario shares one renderer host with the real-time local physics tests.
  workers: 1,
  timeout: 120_000,
  use: { baseURL: site, ignoreHTTPSErrors: tls, viewport: { width: 1440, height: 900 } },
  webServer: [
    {
      command: 'cargo run --quiet --release --locked -p brawler-server',
      url: server + '/healthz',
      env: { BRAWLER_LISTEN: '127.0.0.1:5181', BRAWLER_ORIGIN: site },
      timeout: 120_000,
    },
    {
      command: webCommand + ' -- --port 5180 --strictPort',
      url: site,
      ignoreHTTPSErrors: tls,
      env: { BRAWLER_API_TARGET: server },
    },
  ],
});
