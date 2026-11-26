import * as engine from '../generated/wasm/brawler_wasm';

let initialized = false;

export async function loadPhysics(signal: AbortSignal) {
  signal.throwIfAborted();
  if (!initialized) {
    const response = await fetch(new URL('../generated/wasm/brawler_wasm_bg.wasm', import.meta.url), {signal});
    if (!response.ok) throw new Error('Physics download failed');
    const module = response.headers.get('Content-Type') === 'application/wasm' && WebAssembly.compileStreaming
      ? await WebAssembly.compileStreaming(response)
      : await WebAssembly.compile(await response.arrayBuffer());
    // Compilation may finish after cancellation. Only a live attempt can initialize the shared WASM instance.
    signal.throwIfAborted();
    engine.initSync({module});
    initialized = true;
  }
  return engine;
}
