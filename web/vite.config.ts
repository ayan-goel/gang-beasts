import { defineConfig } from 'vite';

const proxy = { '/api': process.env.BRAWLER_API_TARGET ?? 'http://127.0.0.1:8080' };

export default defineConfig({
  server: { proxy },
  preview: { proxy },
  build: { rollupOptions: { input: ['index.html', 'play/index.html', 'rooms/index.html'] } },
});
