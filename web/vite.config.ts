import { defineConfig } from 'vite';

export default defineConfig({
  build: { rollupOptions: { input: ['index.html', 'play/index.html'] } },
});
