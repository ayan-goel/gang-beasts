import type { Group } from 'three';
import fighterUrl from '../../../../assets/fighters/roof-crew.glb?url';

let model: Group | undefined;

export async function loadFighterModel(signal?: AbortSignal) {
  signal?.throwIfAborted();
  if (model) return model;
  const {GLTFLoader} = await import('three/addons/loaders/GLTFLoader.js');
  const response = await fetch(fighterUrl, {signal});
  if (!response.ok) throw new Error('Fighter download failed');
  const data = await response.arrayBuffer();
  signal?.throwIfAborted();
  const result = await new GLTFLoader().parseAsync(data, new URL('.', response.url).href);
  signal?.throwIfAborted();
  return model ??= result.scene;
}
