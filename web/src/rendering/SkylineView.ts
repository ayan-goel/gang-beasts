import * as THREE from 'three';
import { CameraObstacles } from './CameraObstacles';
import { dressDistrict } from './StationDistrict';
import { SkylineDetails } from './SkylineDetails';

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

const palette = { deck: 0x729a91, vent: 0x338b84, crate: 0xea7548, panel: 0xe4ac55 };
const box = (width: number, height: number, depth: number, color: number) => {
  const mesh = new THREE.Mesh(new THREE.BoxGeometry(width, height, depth), new THREE.MeshStandardMaterial({ color, roughness: 0.78 }));
  mesh.castShadow = true; mesh.receiveShadow = true;
  return mesh;
};
export class SkylineView {
  readonly obstacles = new CameraObstacles();
  readonly root = new THREE.Group();
  private readonly objects: THREE.Mesh<THREE.BoxGeometry, THREE.MeshStandardMaterial>[];
  private readonly panels: { id: number; material: THREE.MeshStandardMaterial }[] = [];
  constructor(map: SkylineMap, deckPrint?: THREE.Texture) {
    this.obstacles.fixed(map.foundation.center,map.foundation.half);
    this.objects = map.objects.map(({ kind, half: [x, y, z], center, id }) => {
      const mesh = box(2 * x, 2 * y, 2 * z, palette[kind]);
      this.obstacles.add([x,y,z],mesh);
      mesh.name = `${kind}-${id}`;
      mesh.position.set(...center); this.root.add(mesh);
      const details = new SkylineDetails();
      if (kind === 'deck') {
        if (deckPrint) {
          mesh.material.map = deckPrint;
          mesh.material.color.setHex(0xffffff);
          const uv = mesh.geometry.attributes.uv!;
          // The roof print occupies the upper face only; the fascia samples plain painted sheet.
          for (let i = 0; i < uv.count; i++) {
            if (mesh.geometry.attributes.normal!.getY(i) < .99) uv.setXY(i,.32,.32);
          }
        }
        for (const side of [-1, 1]) {
          details.add([2 * x, 0.08, 0.08], [0, -0.1, side * z], 0xe6ded1);
          details.add([0.08, 0.08, 2 * z], [side * x, -0.1, 0], 0xe6ded1);
          for (let i = -8; i <= 8; i += 2) details.add([0.75, 0.012, 0.07], [i, y, side * (z - 0.3)], 0xd8c89f);
        }
        // The façade sits inside the roof footprint. Rings and windows outside the
        // arena are scenery and never supply gameplay collision or grip targets.
        const core = map.foundation;
        details.add([2 * core.half[0], 2 * core.half[1], 2 * core.half[2]],
          [core.center[0] - center[0], core.center[1] - center[1], core.center[2] - center[2]], 0xb96e5f);
        for (let i = 0; i < 5; i++) details.add([2 * core.half[0] + 0.04, 0.18, 2 * core.half[2] + 0.04], [0, -2 - i * 3.2, 0], 0x60445a);
        for (let row=0;row<6;row++) for (let bay=-4;bay<=4;bay++) {
          for (const side of [-1,1]) {
            details.add([1.15,1,.07],[bay*1.96,-2.85-row*2.55,side*core.half[2]],0x55495b);
            details.add([.07,1,1.15],[side*core.half[0],-2.85-row*2.55,bay*1.96],0x55495b);
          }
        }
      } else if (kind === 'vent') {
        details.add([2*x+.025,.08,2*z+.025],[0,-y+.1,0],0x45535d);
        for (const cx of [-.57,.57]) {
          details.drum(.52,.55,.075,[cx,y,0],0xe4c794,24);
          details.drum(.47,.47,.045,[cx,y+.052,0],0x3c4e59,24);
          for (let blade=0;blade<6;blade++) {
            const angle=blade*Math.PI/3;
            details.add([.34,.016,.115],[cx+.21*Math.cos(angle),y+.082,.21*Math.sin(angle)],0x76a19a,-angle+.35);
          }
          details.drum(.11,.13,.045,[cx,y+.1,0],0xe4b474);
        }
        for (const side of [-1,1]) for (let i=-4;i<=4;i++)
          details.add([.12,.53,.035],[i*.23,.04,side*z],0x42696a);
      } else if (kind === 'crate') {
        for (const side of [-1,1]) {
          details.add([2*x+.025,.07,2*z+.025],[0,side*(y-.045),0],0x614955);
          details.add([.083,2*y,2*z+.026],[side*x*.73,0,0],0xffcc84);
          for (const face of [-1,1]) details.add([.14,.12,.034],[side*x*.72,y*.48,face*z],0x584754);
        }
        details.add([.31,.08,.04],[0,.12,z],0x473e4c);
      } else {
        for (let i=-7;i<=7;i++) details.add([1.52,.016,.09],[0,y,i*.5],0x6e5650,-.24);
        for (const side of [-1,1]) details.add([.12,.035,2*z],[side*(x-.08),y,0],0xffdb8a);
        this.panels.push({ id, material: mesh.material });
      }
      details.attach(mesh);
      return mesh;
    });
    dressDistrict(this.root, this.obstacles);
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
