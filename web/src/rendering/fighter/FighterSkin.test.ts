import { readFileSync } from 'node:fs';
import { expect, it } from 'vitest';
import { Euler, Group, Matrix4, MeshBasicMaterial, MeshStandardMaterial, Quaternion, SkinnedMesh, Vector3 } from 'three';
import { GLTFLoader } from 'three/addons/loaders/GLTFLoader.js';
import type { RigDefinition } from '../../game/rigDefinition';
import { FighterSkin } from './FighterSkin';
import { suitMaterial } from './palette';

const rig: RigDefinition = JSON.parse(readFileSync(new URL('../../../../config/character.json', import.meta.url), 'utf8'));
const load = async () => {
  const bytes = new Uint8Array(readFileSync(new URL('../../../../assets/fighters/roof-crew.glb', import.meta.url)));
  return (await new GLTFLoader().parseAsync(bytes.buffer, '')).scene;
};

it('skins an original shared mesh independently to each fighter across full rotations and transformed scene roots', async () => {
  const model = await load();
  const first = new FighterSkin(model, rig, '#f38e66', 0), second = new FighterSkin(model, rig, '#64b7dd', 1);
  expect(first.mesh.geometry).toBe(second.mesh.geometry);
  expect(first.mesh.skeleton).not.toBe(second.mesh.skeleton);
  expect(first.mesh.material).not.toBe(second.mesh.material);
  const {geometry, skeleton} = first.mesh;
  const silhouette = first.mesh.children.find(child => child instanceof SkinnedMesh)!;
  expect(silhouette.geometry).toBe(geometry);
  expect(silhouette.skeleton).toBe(skeleton);
  expect(silhouette.castShadow).toBe(false);
  expect(skeleton.bones.map(b => b.name)).toEqual(rig.bodies.map(b => b.part));
  const weights = geometry.getAttribute('skinWeight');
  expect(Array.from({length:weights.count}, (_, i) => weights.getY(i)).some(w => w > .1 && w < .9)).toBe(true);
  const parent = new Group();
  parent.position.set(-3, 1, 7);
  parent.rotation.y = 1.3;
  parent.scale.setScalar(1.6);
  parent.add(first.root);
  const pose = new Float32Array(15 * 14);
  for (const turn of [0, Math.PI / 2, Math.PI, 2 * Math.PI]) {
    const expected: Matrix4[] = [];
    rig.bodies.forEach((body, index) => {
      const q = new Quaternion().setFromEuler(new Euler(index * .17, turn, index * -.12));
      const pos = new Vector3(...body.position).add(new Vector3(4, 2, -3));
      pos.toArray(pose, index*14); q.toArray(pose, index*14+3);
      const bind = skeleton.boneInverses[index]!.clone().invert();
      const local = new Matrix4().makeTranslation(-body.position[0], -body.position[1], -body.position[2]).multiply(bind);
      expected.push(new Matrix4().compose(pos,q,new Vector3(1,1,1)).multiply(local));
    });
    first.update(pose);
    parent.updateMatrixWorld(true);
    for (const [index,bone] of skeleton.bones.entries()) {
      expect(Math.max(...bone.matrixWorld.elements.map((v,i) => Math.abs(v - expected[index]!.elements[i]!)))).toBeLessThan(1e-5);
    }
    const indices = geometry.getAttribute('skinIndex'), positions = geometry.getAttribute('position');
    let maxError = 0;
    for (let i = 0; i < weights.count; i++) {
      const vertex = first.mesh.getVertexPosition(i, new Vector3());
      const world = first.mesh.localToWorld(vertex);
      if (i % 97 === 0) {
        const hidden = silhouette.localToWorld(silhouette.getVertexPosition(i, new Vector3()));
        expect(hidden.distanceTo(world)).toBeLessThan(1e-5);
      }
      const rest = new Vector3().fromBufferAttribute(positions,i);
      const weighted = new Vector3();
      for (let j = 0; j < 4; j++) {
        const w = weights.getComponent(i,j);
        if (!w) continue;
        const k = indices.getComponent(i,j);
        const relative = rest.clone().sub(new Vector3(...rig.bodies[k]!.position));
        relative.applyQuaternion(new Quaternion().fromArray(pose,k*14+3));
        relative.add(new Vector3().fromArray(pose,k*14));
        weighted.addScaledVector(relative,w);
      }
      maxError = Math.max(maxError,world.distanceTo(weighted));
    }
    expect(maxError).toBeLessThan(1e-5);

  }
  expect(first.radii.every(radius => radius > 0 && radius < .5)).toBe(true);
  first.dispose(); second.dispose();
});

it('paints eight distinct wristband patterns when every kit uses the same suit color', async () => {
  const model = await load();
  const skin = new FighterSkin(model, rig, '#999999', 0);
  const uv = skin.mesh.geometry.getAttribute('uv');
  const paint = new Set(Array.from({length:uv.count}, (_,i) => Math.floor(16*uv.getX(i))));
  const patterns = new Set<string>();
  for (let kit = 0; kit < 8; kit++) {
    const material = suitMaterial('#999999',kit);
    const bytes = material.map!.image.data as Uint8Array;
    const markings = [12,13,14,15].map(index => {
      expect(paint.has(index)).toBe(true);
      return bytes[4*index];
    });
    patterns.add(markings.join(','));
    material.map!.dispose();
    material.dispose();
  }
  expect(patterns.size).toBe(8);
  skin.dispose();
});

it('repaints a prepared suit and its occluded silhouette without replacing the skin or texture', async () => {
  const skin = new FighterSkin(await load(), rig, '#f38e66', 0);
  const original = skin.mesh.material;
  if (!(original instanceof MeshStandardMaterial)) throw new Error('Missing suit material');
  const texture = original.map!;
  const expected = suitMaterial('#e69eb9', 5);
  skin.setKit('#e69eb9', 5);
  expect(skin.mesh.material).toBe(original);
  expect(original.map).toBe(texture);
  expect(texture.image.data).toEqual(expected.map.image.data);
  const outline = skin.mesh.children.find(child => child instanceof SkinnedMesh)!;
  expect(outline.material).toBeInstanceOf(MeshBasicMaterial);
  expect((outline.material as MeshBasicMaterial).color.getHex()).toBe(0xe69eb9);
  expected.map.dispose();
  expected.dispose();
  skin.dispose();
});

it('gives the bean a broad head and low belly at gameplay scale', async () => {
  const skin = new FighterSkin(await load(), rig, '#ff533f', 0);
  const {position, skinIndex} = skin.mesh.geometry.attributes;
  let headWidth = 0, bellyBottom = Infinity;
  for (let i = 0; i < position!.count; i++) {
    const part = rig.bodies[skinIndex!.getX(i)]!.part;
    if (part === 'head') headWidth = Math.max(headWidth, Math.abs(position!.getX(i)) * 2);
    if (part === 'pelvis') bellyBottom = Math.min(bellyBottom, position!.getY(i));
  }
  expect(headWidth).toBeGreaterThan(.53);
  expect(bellyBottom).toBeLessThan(.51);
  skin.dispose();
});
