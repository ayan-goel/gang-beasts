import { readFileSync } from 'node:fs';
import { defineConfig, type Connect } from 'vite';

const target = process.env.BRAWLER_API_TARGET ?? 'http://127.0.0.1:8080';
const cert = process.env.BRAWLER_TLS_CERT, key = process.env.BRAWLER_TLS_KEY;
if (Boolean(cert) !== Boolean(key)) throw new Error('Set both BRAWLER_TLS_CERT and BRAWLER_TLS_KEY to enable HTTPS.');
const https = cert && key ? {cert:readFileSync(cert), key:readFileSync(key)} : undefined;
const proxy = { '/api': target, '/ws': { target, ws: true } };
const gamePages = new Set(['/play', '/rooms', '/playground']);

const directoryIndex: Connect.NextHandleFunction = (req, res, next) => {
  const url = req.url ?? '';
  const path = url.split('?', 1)[0]!;
  if ((req.method === 'GET' || req.method === 'HEAD') && gamePages.has(path)) {
    // Preserve invite codes when normalizing a bookmark before Vite's root-page fallback.
    res.writeHead(308, {Location: path + '/' + url.slice(path.length)});
    res.end();
    return;
  }
  next();
};

export default defineConfig({
  plugins: [{
    name: 'game-directory-index',
    configureServer(server) { server.middlewares.use(directoryIndex); },
    configurePreviewServer(server) { server.middlewares.use(directoryIndex); },
  }],
  server: { proxy, https },
  preview: { proxy, https },
  build: { rollupOptions: { input: ['index.html', 'play/index.html', 'rooms/index.html', 'playground/index.html'] } },
});
