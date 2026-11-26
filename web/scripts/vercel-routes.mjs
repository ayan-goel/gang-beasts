import { mkdirSync, writeFileSync } from 'node:fs';

const raw = process.env.VITE_BACKEND_URL?.trim();
const backend = URL.parse(raw ?? '');
if (!backend || backend.protocol !== 'https:' || backend.href !== backend.origin + '/') {
  throw new Error('Set VITE_BACKEND_URL to the public Railway HTTPS origin without a path.');
}

// Vercel imports the project before running its build. Resolve the external destination here.
const routes = [
  { src: '^/api/(.*)$', dest: backend.origin + '/api/$1' },
  { src: '^/(play|rooms|playground)/?$', dest: '/$1/index.html' },
  { src: '^/$', dest: '/index.html' },
  { handle: 'filesystem' },
];
mkdirSync('.vercel/output', { recursive: true });
writeFileSync('.vercel/output/config.json', JSON.stringify({ version: 3, routes }, null, 2) + '\n');
