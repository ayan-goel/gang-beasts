import type { Group } from 'three';
import fighterUrl from '../../../../assets/fighters/roof-crew.glb?url';

let loading: Promise<Group> | undefined;

export function loadFighterModel() {
  loading ??= import('three/addons/loaders/GLTFLoader.js')
    .then(async ({GLTFLoader}) => (await new GLTFLoader().loadAsync(fighterUrl)).scene)
    .catch(error => { loading = undefined; throw error; });
  return loading;
}
