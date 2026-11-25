import { defineConfig } from '@playwright/test';

const site = 'http://127.0.0.1:5180';
const server = 'http://127.0.0.1:5181';

export default defineConfig({
  testDir: './tests',
  timeout: 120_000,
  use: { baseURL: site, viewport: { width: 1440, height: 900 } },
  webServer: [
    {
      command: 'cargo run --quiet --release --locked -p brawler-server',
      url: server + '/healthz',
      env: { BRAWLER_LISTEN: '127.0.0.1:5181', BRAWLER_ORIGIN: site },
      timeout: 120_000,
    },
    {
      command: 'npm run dev -- --port 5180 --strictPort',
      url: site,
      env: { BRAWLER_API_TARGET: server },
    },
  ],
});
