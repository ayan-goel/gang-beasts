export const physicsUnsupported = 'Skyline needs WebAssembly to run its physics. Open the game in a browser with WebAssembly enabled.';

export function supportsPhysics() {
  return typeof WebAssembly !== 'undefined' && typeof WebAssembly.instantiate === 'function';
}
