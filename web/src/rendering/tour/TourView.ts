import {
  Color, Float32BufferAttribute, Group, Mesh, MeshStandardMaterial, type Texture,
} from 'three';
import { RoundedBoxGeometry } from 'three/addons/geometries/RoundedBoxGeometry.js';
import { CameraObstacles } from '../CameraObstacles';
import type { ArenaObject, MatchView, SkylineMap } from '../SkylineView';
import { deckShape } from './DeckShape';
import { ArenaWater } from './Water';
import { dressProp } from './Props';
import { dressSurroundings, type Mobile } from './Surroundings';
import { stagePalette, type StagePalette } from './palette';

function bodyMaterial(obj: ArenaObject, p: StagePalette, texture?: Texture) {
  const [x,y,z] = obj.half;
  const deck = obj.kind === 'deck' || obj.kind === 'panel';
  const geometry = deckShape(obj) ?? new RoundedBoxGeometry(2*x,2*y,2*z,1,Math.min(x,y,z,.12)*.68);
  const {position,normal,uv} = geometry.attributes;
  const paint = new Float32Array(position!.count*3);
  const side = new Color(obj.kind === 'vent' ? p.accent : p.edge);
  const face = new Color(deck && texture ? 0xffffff : p.deck);
  if (obj.art === 'sandbar') { face.set(0xf7cd87); side.set(0xbe986a); }
  if (obj.art === 'basalt') { face.set(0x8fb26a); side.set(0x7a665b); }
  if (['floe','radio-floe','ridge','snow-shelf'].includes(obj.art ?? '')) {
    face.set(0xc6ebe7); side.set(0x72b7ca);
  }
  if (obj.art === 'track') face.set(0x6c727e);
  if (obj.art === 'train' || obj.art === 'luggage') side.set(p.accent);
  const tint = new Color();
  for (let i=0;i<position!.count;i++) {
    const nx = normal!.getX(i), ny = normal!.getY(i);
    const px = position!.getX(i), py = position!.getY(i), pz = position!.getZ(i);
    // World-scale UVs keep ice cracks and rivets the same size on both a narrow bridge and a wide deck.
    if (Math.abs(ny) > .65) uv!.setXY(i,px/3.2,pz/3.2);
    else if (Math.abs(nx) > .65) uv!.setXY(i,pz/3.2,py/3.2);
    else uv!.setXY(i,px/3.2,py/3.2);
    tint.copy(side).lerp(face,Math.max(0,ny)**4).toArray(paint,3*i);
  }
  geometry.setAttribute('color',new Float32BufferAttribute(paint,3));
  return new Mesh(geometry,new MeshStandardMaterial({map:deck ? texture ?? null : null,vertexColors:true,roughness:.68}));
}

export class TourView {
  readonly root = new Group();
  readonly obstacles = new CameraObstacles();
  private readonly objects: Group[] = [];
  private readonly warnings: {material:MeshStandardMaterial; starts:number[]}[] = [];
  private readonly mobiles: Mobile[];
  private readonly water?: ArenaWater;
  private elapsed = 0;

  constructor(map: SkylineMap, texture?: Texture) {
    const {id,tour} = map;
    if (!id || !tour) throw new Error('Missing tour art direction');
    const palette = stagePalette(id);
    for (const obj of map.objects) {
      const group = new Group();
      group.position.fromArray(obj.center);
      group.name = obj.art ?? obj.kind;
      this.root.add(group);
      this.obstacles.add(obj.half,group);
      if (!['shark','palm','lava-vent','tail-fin','cockpit','beacon'].includes(obj.art ?? '')) {
        const body = bodyMaterial(obj,palette,texture);
        body.receiveShadow = body.castShadow = true;
        group.add(body);
        const starts = tour.motions.filter(motion => motion.object === obj.id).map(motion => motion.start);
        if (starts.length) this.warnings.push({material:body.material,starts});
      }
      dressProp(group,obj,palette);
      this.objects.push(group);
    }
    this.mobiles = dressSurroundings(this.root,palette);
    if (tour.water != null) {
      this.water = new ArenaWater(tour.water,palette.water,palette.haze,map.objects);
      this.root.add(this.water.mesh);
    }
  }

  update(poses: Float32Array, _hazard: MatchView['hazard'], dt = 0, players: Float32Array[] = [], match?: MatchView) {
    if (poses.length !== this.objects.length*8) throw new Error('Invalid tour arena pose buffer');
    this.objects.forEach((group,i)=>{
      group.position.fromArray(poses,i*8);
      group.quaternion.fromArray(poses,i*8+3);
      group.visible = poses[i*8+7] === 1;
    });
    this.elapsed += Math.max(0,Math.min(.12,dt));
    this.water?.update(this.elapsed,players,match?.visible ?? [],poses);
    for (const mobile of this.mobiles) mobile.group.rotation[mobile.axis] = this.elapsed*mobile.speed;
    const tick = match?.arenaTick;
    for (const {material,starts} of this.warnings) {
      const warning = tick !== undefined && match?.phase === 'playing' &&
        starts.some(start => tick >= start - 120 && tick < start);
      material.emissive.setHex(warning ? 0xff9e42 : 0);
      material.emissiveIntensity = warning ? .20+.10*Math.sin(tick! * Math.PI/30) : 0;
    }
  }
}
