#!/usr/bin/env bash
set -euo pipefail

cd "$(dirname "$0")/../web"
node scripts/vercel-routes.mjs
cd ..

export PATH="${CARGO_HOME:-$HOME/.cargo}/bin:$PATH"
if ! command -v rustup >/dev/null; then
  curl --proto '=https' --tlsv1.2 -fsSL https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain none
fi

# Native and browser builds include the compiler version in their multiplayer handshake.
rustup show active-toolchain >/dev/null
rustup target add wasm32-unknown-unknown
cargo install wasm-pack --version 0.13.1 --locked --root .tools
npm --prefix web run build -- --outDir .vercel/output/static --emptyOutDir
