import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { mkdtempSync, readFileSync, rmSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { test } from 'node:test';

test('the web project imports before a backend URL exists and builds a concrete API route', () => {
  const config = JSON.parse(readFileSync(new URL('../vercel.json', import.meta.url)));
  assert.equal(config.framework, 'vite');
  assert.equal(config.rewrites, undefined);
  const root = mkdtempSync(join(tmpdir(), 'skyline-vercel-'));
  try {
    const script = fileURLToPath(new URL('./vercel-routes.mjs', import.meta.url));
    const backend = 'https://test.up.railway.app';
    execFileSync(process.execPath, [script], { cwd: root, env: { VITE_BACKEND_URL: backend } });
    const output = JSON.parse(readFileSync(join(root, '.vercel/output/config.json')));
    assert.equal(output.version, 3);
    const resolve = path => {
      const route = output.routes.find(r => r.src && new RegExp(r.src).test(path));
      return route && path.replace(new RegExp(route.src), route.dest);
    };
    assert.equal(resolve('/api/rooms/ABCDEF/join'), backend + '/api/rooms/ABCDEF/join');
    for (const page of ['rooms', 'play', 'playground']) {
      assert.equal(resolve('/' + page + '/'), '/' + page + '/index.html');
      assert.equal(resolve('/' + page), '/' + page + '/index.html');
    }
    assert.equal(resolve('/'), '/index.html');
    assert.equal(resolve('/assets/game.wasm'), undefined);
    assert.throws(() => execFileSync(process.execPath, [script], {
      cwd: root, env: { VITE_BACKEND_URL: backend + '/api' }, stdio: 'pipe',
    }), /VITE_BACKEND_URL/);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
});
