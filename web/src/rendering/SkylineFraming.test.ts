import {readFileSync} from 'node:fs';
import {PerspectiveCamera, Quaternion, Vector3} from 'three';
import {expect, test} from 'vitest';
import type {RigDefinition} from '../game/rigDefinition';
import {GroupCamera} from './GroupCamera';
import {geometryForBody} from './RagdollView';

const rig = JSON.parse(readFileSync(new URL('../../../config/character.json',import.meta.url),'utf8')) as RigDefinition;
function fighter(x: number, y: number, z: number) {
  const poses = new Float32Array(rig.bodies.length * 14);
  for (let i = 0; i < rig.bodies.length; i++) {
    const position = rig.bodies[i]!.position;
    const rotation = new Quaternion().setFromAxisAngle(new Vector3(1,1,0).normalize(),i * .4);
    poses.set([x + position[0],y + position[1],z + position[2], ...rotation.toArray()], 14*i);
  }
  return poses;
}

test('the complete drawn rigs fit the central 70 percent throughout turns and a viewport resize', () => {
  const group = new GroupCamera(rig.bodies);
  const camera = new PerspectiveCamera(45,.5,.1,150);
  const players = [fighter(-10,-7,-6),fighter(10,1,8)];
  const vertices = rig.bodies.map(({shape}) => {
    const geometry = geometryForBody(shape);
    const positions = geometry.getAttribute('position');
    const points = Array.from({length:positions.count}, (_,i) => new Vector3().fromBufferAttribute(positions,i));
    geometry.dispose();
    return points;
  });
  const point = new Vector3(), center = new Vector3(), rotation = new Quaternion();
  for (let view = 0; view < 8; view++) {
    group.turn(1);
    for (let frame = 0; frame < 60; frame++) {
      camera.aspect = frame < 30 ? .45 : 1.85;
      camera.updateProjectionMatrix();
      group.update(camera,players,false,1/60,players[1]);
      if (![0,12,30,59].includes(frame)) continue;
      camera.updateMatrixWorld();
      let edge = 0, depth = 0;
      for (const poses of players) for (const [i,body] of vertices.entries()) {
        center.fromArray(poses,i*14); rotation.fromArray(poses,i*14+3);
        for (const v of body) {
          point.copy(v).applyQuaternion(rotation).add(center).project(camera);
          edge = Math.max(edge, Math.abs(point.x), Math.abs(point.y));
          depth = Math.max(depth, Math.abs(point.z));
        }
      }
      expect(edge).toBeLessThanOrEqual(.70001);
      expect(depth).toBeLessThan(1);
    }
  }
});

test('online focus shifts toward the owned fighter and drops its preference when that fighter is out', () => {
  const players = [fighter(-8,0,0),fighter(8,0,0)];
  const center = (visible: Float32Array[], owner?: Float32Array) => {
    const group = new GroupCamera(rig.bodies);
    const camera = new PerspectiveCamera(45,1,.1,150);
    group.update(camera,visible,false,1/60,owner);
    const direction = camera.getWorldDirection(new Vector3());
    const t = -camera.position.z / direction.z;
    return camera.position.x + t*direction.x;
  };
  expect(center(players)).toBeCloseTo(0);
  expect(center(players,players[1])).toBeGreaterThan(.1);
  expect(center(players,players[1])).toBeLessThan(3);
  expect(center([players[0]!],players[1])).toBeCloseTo(-8);
});

test('an empty, narrow arena overview keeps the far roof corners inside the clipping range', () => {
  const group = new GroupCamera(rig.bodies);
  const camera = new PerspectiveCamera(45,.2,.1,150);
  for (let i=0; i<8; i++) {
    group.turn(1);
    for (let frame=0; frame<60; frame++) group.update(camera,[],false,1/60);
    camera.updateMatrixWorld();
    for (const x of [-12,12]) for (const z of [-10,10]) {
      const corner = new Vector3(x,0,z).project(camera);
      expect(Math.max(Math.abs(corner.x),Math.abs(corner.y))).toBeLessThan(.7);
      expect(Math.abs(corner.z)).toBeLessThan(1);
    }
  }
});
