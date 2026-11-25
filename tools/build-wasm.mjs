import { execFileSync } from 'node:child_process';
import { copyFileSync, mkdirSync } from 'node:fs';
import { fileURLToPath } from 'node:url';

const root = fileURLToPath(new URL('../', import.meta.url));
execFileSync(`${root}.tools/bin/wasm-pack`, [
  'build', 'crates/wasm', '--target', 'web', '--release',
  '--out-dir', '../../web/src/generated/wasm', '--', '--locked',
], { cwd: root, stdio: 'inherit' });
mkdirSync(`${root}web/src/generated`, { recursive: true });
copyFileSync(`${root}config/physics.json`, `${root}web/src/generated/physics.json`);
