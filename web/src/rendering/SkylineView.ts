import * as THREE from 'three';

type Kind = 'deck' | 'vent' | 'crate' | 'panel';
export interface SkylineMap {
  foundation: { center: [number, number, number]; half: [number, number, number] };
  objects: { id: number; kind: Kind; center: [number, number, number]; half: [number, number, number] }[];
}
export interface MatchView {
  phase: 'countdown' | 'playing' | 'round_results' | 'match_results';
  round: number; epoch: string; remaining: number; scores: number[]; winner: number | null;
  timeLeft: number; visible: boolean[];
  hazard: 'quiet' | 'west_warning' | 'east_warning' | 'gust_warning' | 'gust';
}

const palette = { deck: 0x586f79, vent: 0xb1bfad, crate: 0xe1a568, panel: 0xdbad56 };
const box = (width: number, height: number, depth: number, color: number) => {
  const mesh = new THREE.Mesh(new THREE.BoxGeometry(width, height, depth), new THREE.MeshStandardMaterial({ color, roughness: 0.78 }));
  mesh.castShadow = true; mesh.receiveShadow = true;
  return mesh;
};
function trim(parent: THREE.Object3D, size: [number, number, number], position: [number, number, number], color: number) {
  const mesh = box(...size, color); mesh.position.set(...position); parent.add(mesh); return mesh;
}

export class SkylineView {
  readonly root = new THREE.Group();
  private readonly objects: THREE.Mesh<THREE.BoxGeometry, THREE.MeshStandardMaterial>[];
  private readonly panels: { id: number; material: THREE.MeshStandardMaterial }[] = [];
  constructor(map: SkylineMap) {
    this.objects = map.objects.map(({ kind, half: [x, y, z], center, id }) => {
      const mesh = box(2 * x, 2 * y, 2 * z, palette[kind]);
      mesh.name = `${kind}-${id}`;
      mesh.position.set(...center); this.root.add(mesh);
      if (kind === 'deck') {
        for (const side of [-1, 1]) {
          trim(mesh, [2 * x, 0.08, 0.08], [0, -0.1, side * z], 0xe6ded1);
          trim(mesh, [0.08, 0.08, 2 * z], [side * x, -0.1, 0], 0xe6ded1);
          for (let i = -8; i <= 8; i += 2) trim(mesh, [0.75, 0.012, 0.07], [i, y, side * (z - 0.3)], 0xd8c89f);
        }
        // The façade sits inside the roof footprint. Rings and windows outside the
        // arena are scenery and never supply gameplay collision or grip targets.
        const core = map.foundation;
        trim(mesh, [2 * core.half[0], 2 * core.half[1], 2 * core.half[2]],
          [core.center[0] - center[0], core.center[1] - center[1], core.center[2] - center[2]], 0x354654);
        for (let i = 0; i < 5; i++) trim(mesh, [2 * core.half[0] + 0.04, 0.18, 2 * core.half[2] + 0.04], [0, -2 - i * 3.2, 0], 0x75898b);
      } else if (kind === 'vent') {
        for (let i = -3; i <= 3; i++) trim(mesh, [0.14, 0.025, 1.05], [i * 0.29, y, 0], 0x465960);
        trim(mesh, [2 * x + 0.035, 0.08, 2 * z + 0.035], [0, -y + 0.08, 0], 0x596b6e);
      } else if (kind === 'crate') {
        for (const side of [-1, 1]) trim(mesh, [0.1, 2 * y + 0.012, 2 * z + 0.012], [side * x * 0.53, 0, 0], 0x74584b);
      } else {
        for (let i = -3; i <= 3; i++) trim(mesh, [1.5, 0.016, 0.14], [0, y, i], 0x65533e);
        this.panels.push({ id, material: mesh.material });
      }
      return mesh;
    });
    const buildings = [[-25, -22, 7, 21], [-12, -31, 6, 16], [0, -38, 8, 24], [14, -30, 6, 19], [29, -25, 8, 28], [-31, -4, 6, 18], [34, -4, 6, 15], [-34, 15, 9, 22], [33, 22, 8, 16], [-20, 35, 7, 20]];
    for (const [x, z, w, h] of buildings as [number, number, number, number][]) {
      const tower = trim(this.root, [w, h, w], [x, -24 + h / 2, z], 0x647c8a);
      for (let row = 1; row < h / 2; row += 2) trim(tower, [w + 0.05, 0.28, w + 0.05], [0, h / 2 - row * 1.6, 0], 0x849da3);
      trim(tower, [w * 0.38, 1.5, w * 0.38], [0, h / 2 + 0.75, 0], 0x718997);
    }
  }
  update(poses: Float32Array, hazard: MatchView['hazard']) {
    if (poses.length !== this.objects.length * 8) throw new Error('Invalid Skyline pose buffer');
    this.objects.forEach((mesh, i) => {
      mesh.position.fromArray(poses, i * 8); mesh.quaternion.fromArray(poses, i * 8 + 3); mesh.visible = poses[i * 8 + 7] === 1;
    });
    for (const panel of this.panels) {
      const warned = hazard === 'west_warning' && panel.id === 8 || hazard === 'east_warning' && panel.id === 9;
      panel.material.emissive.setHex(warned ? 0xad3500 : 0);
    }
  }
}
