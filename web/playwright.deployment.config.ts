import { defineConfig } from '@playwright/test';

const frontend = 'http://127.0.0.1:24371';
const backend = 'http://localhost:24372';

export default defineConfig({
  testDir: './deployment-tests',
  workers: 1,
  timeout: 120000,
  metadata: { backend },
  use: { baseURL: frontend },
  webServer: [
    {
      command: 'cargo run --quiet --release --locked -p brawler-server',
      env: { PORT: '24372', BRAWLER_ORIGIN: frontend },
      url: 'http://127.0.0.1:24372/healthz',
      timeout: 120000,
    },
    {
      command: 'npm run build -- --outDir ../.cache/hosting-split-dist && npm run preview -- --outDir ../.cache/hosting-split-dist --port 24371 --strictPort',
      env: { VITE_BACKEND_URL: backend, BRAWLER_API_TARGET: 'http://127.0.0.1:24372' },
      url: frontend,
      timeout: 120000,
    },
  ],
});
