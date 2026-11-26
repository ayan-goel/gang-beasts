import { Bone, Matrix4, Quaternion, SkinnedMesh, Vector3, type Object3D } from 'three';
import { clone } from 'three/addons/utils/SkeletonUtils.js';
import type { RigDefinition } from '../../game/rigDefinition';
import { paintSuit, suitMaterial } from './palette';
import { addOccludedSilhouette } from './occludedSilhouette';

const unit = new Vector3(1,1,1);

export class FighterSkin {
  readonly root: Object3D;
  readonly mesh: SkinnedMesh;
  readonly radii: number[];
  private readonly material;
  private readonly silhouette;
  private readonly bindings;
  private readonly p = new Vector3();
  private readonly q = new Quaternion();

  constructor(model: Object3D, rig: RigDefinition, team: string, kit: number) {
    this.root = clone(model);
    const mesh = this.root.getObjectByName('RoofCrew');
    if (!(mesh instanceof SkinnedMesh)) throw new Error('Skyline fighter skin is missing');
    this.mesh = mesh;
    const {bones, boneInverses} = mesh.skeleton;
    if (bones.length !== rig.bodies.length) throw new Error('Invalid Skyline fighter skeleton');
    this.bindings = rig.bodies.map((body, i) => {
      const bone = bones[i]!;
      if (bone.name !== body.part || !bone.parent) throw new Error('Skyline fighter bone mismatch');
      bone.matrixAutoUpdate = false;
      const [x, y, z] = body.position;
      const offset = new Matrix4().makeTranslation(-x,-y,-z).multiply(boneInverses[i]!.clone().invert());
      return {bone, parent:bone.parent instanceof Bone ? bones.indexOf(bone.parent) : -1, offset, world:new Matrix4()};
    });
    this.radii = mesh.geometry.userData.bodyRadii ??= visualRadii(mesh, rig);
    this.material = suitMaterial(team, kit);
    mesh.material = this.material;
    mesh.castShadow = mesh.receiveShadow = true;
    // A bind-pose sphere at the origin cannot cull a fighter moving across the roof.
    mesh.frustumCulled = false;
    this.silhouette = addOccludedSilhouette(mesh, team);
  }

  setKit(team: string, kit: number) {
    paintSuit(this.material.map, team, kit);
    this.silhouette.color.set(team);
  }

  update(states: Float32Array) {
    if (states.length !== this.bindings.length * 14) throw new Error('Invalid fighter pose');
    for (let i = 0; i < this.bindings.length; i++) {
      const binding = this.bindings[i]!;
      this.p.fromArray(states, i*14);
      this.q.fromArray(states, i*14+3);
      binding.world.compose(this.p,this.q,unit).multiply(binding.offset);
    }
    for (const {bone, parent, world} of this.bindings) {
      if (parent < 0) bone.parent!.updateWorldMatrix(true,false);
      const parentWorld = parent < 0 ? bone.parent!.matrixWorld : this.bindings[parent]!.world;
      bone.matrix.copy(parentWorld).invert().multiply(world);
      bone.matrixWorldNeedsUpdate = true;
    }
  }

  dispose() {
    this.silhouette.dispose();
    this.mesh.skeleton.dispose();
    this.material.map?.dispose();
    this.material.dispose();
  }
}

function visualRadii(mesh: SkinnedMesh, rig: RigDefinition) {
  const radii = rig.bodies.map(() => 0);
  const {position, skinIndex, skinWeight} = mesh.geometry.attributes;
  const p = new Vector3();
  const centers = rig.bodies.map(body => new Vector3(...body.position));
  for (let v = 0; v < position!.count; v++) {
    p.fromBufferAttribute(position!, v).applyMatrix4(mesh.bindMatrix);
    for (let influence = 0; influence < 4; influence++) {
      if (!skinWeight!.getComponent(v,influence)) continue;
      const bone = skinIndex!.getComponent(v,influence);
      radii[bone] = Math.max(radii[bone]!, p.distanceTo(centers[bone]!));
    }
  }
  return radii;
}
