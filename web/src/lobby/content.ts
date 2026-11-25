import type { RigDefinition } from '../game/rigDefinition';
import type { SkylineMap } from '../rendering/SkylineView';

export interface SkylineContent {
  map: SkylineMap & { schedule: { warn_tick: number; west_tick: number; east_tick: number; gust_tick: number; end_tick: number } };
  fighter: RigDefinition;
  engineHash: string;
  mapHash: string;
}

let cached: SkylineContent | undefined;

export async function loadSkyline(): Promise<SkylineContent> {
  if (cached) return cached;
  const wasm = await import('../generated/wasm/brawler_wasm');
  await wasm.default();
  const content: SkylineContent = JSON.parse(wasm.skyline_content());
  const digest = /^[a-f0-9]{64}$/;
  if (!digest.test(content.engineHash) || !digest.test(content.mapHash) || content.fighter.bodies.length !== 15) {
    throw new Error('Invalid Skyline build');
  }
  cached = content;
  return content;
}

export function loadedPacket(content: SkylineContent) {
  const packet = new Uint8Array(130);
  packet.set([20, 1]);
  packet.set(new TextEncoder().encode(content.engineHash + content.mapHash), 2);
  return packet;
}
