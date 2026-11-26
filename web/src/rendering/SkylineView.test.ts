import { readFileSync } from 'node:fs';
import { Color, Mesh, Vector3, type BufferGeometry, type MeshStandardMaterial } from 'three';
import { expect, it } from 'vitest';
import { SkylineView } from './SkylineView';

const map = JSON.parse(readFileSync(new URL('../../../assets/maps/skyline/map.json', import.meta.url), 'utf8'));

it('draws static details together while each physical object and warning remains independent', () => {
  const view = new SkylineView(map);
  const meshes: Mesh[] = [];
  view.root.traverse(node => { if (node instanceof Mesh) meshes.push(node); });
  expect(meshes.length).toBeLessThanOrEqual(2 * map.objects.length + 1);
  const deck = view.root.getObjectByName('deck-1')!;
  const details = deck.children[0] as Mesh;
  const pos = details.geometry.getAttribute('position'), colors = details.geometry.getAttribute('color');
  const trimColor = new Color(0xe6ded1), point = new Vector3();
  const trimVertex = Array.from({length:pos.count}, (_,i)=>i).some(i => {
    point.fromBufferAttribute(pos, i);
    return Math.abs(point.x - 10) < 1e-5 && Math.abs(point.z - 10.04) < 1e-5
      && Math.abs(colors.getX(i) - trimColor.r) < 1e-5;
  });
  expect(trimVertex).toBe(true);
  const poses = new Float32Array(map.objects.length * 8);
  for (let i = 0; i < map.objects.length; i++) poses.set([i,2,3,0,0,0,1,1], i*8);
  view.update(poses,'west_warning');
  type Panel = Mesh<BufferGeometry, MeshStandardMaterial>;
  const west = view.root.getObjectByName('panel-8') as Panel, east = view.root.getObjectByName('panel-9') as Panel;
  expect(west.position.x).toBe(7);
  expect(west.material.emissive.getHex()).toBe(0xad3500);
  expect(east.material.emissive.getHex()).toBe(0);
  view.update(poses,'east_warning');
  expect(west.material.emissive.getHex()).toBe(0);
  expect(east.material.emissive.getHex()).toBe(0xad3500);
});
