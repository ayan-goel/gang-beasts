import { expect, test } from 'vitest';
import { Object3D, PerspectiveCamera, Vector3 } from 'three';
import { CameraObstacles } from './CameraObstacles';
import { GroupCamera } from './GroupCamera';

function fighter(x: number, z: number) {
  const pose = new Float32Array(15*14);
  for (let i=0; i<15; i++) pose.set([x,0.8,z], i*14);
  return pose;
}

test('the swept lens sees a grazing corner and follows a tilted or retired walkway', () => {
  const obstacles = new CameraObstacles();
  const panel = new Object3D();
  panel.position.set(2,1,5);
  obstacles.add([2,.2,.25],panel);
  const up = new Vector3(0,0,1), eye = new Vector3(-.1,1,0);
  expect(obstacles.cast(eye,up,8,0).distance).toBe(8);
  expect(obstacles.cast(eye,up,8,.2).distance).toBeCloseTo(4.55);
  panel.rotation.y = Math.PI/2;
  expect(obstacles.cast(eye,up,8,.2).distance).toBe(8);
  panel.position.x = 0;
  expect(obstacles.cast(eye,up,8,.2).distance).toBeCloseTo(2.8);
  panel.visible = false;
  expect(obstacles.cast(eye,up,8,.2).distance).toBe(8);
});

test('an obstruction at the normal pitch selects a clear group view and preserves the control heading', () => {
  const wall = new Object3D();
  wall.position.set(0,1.75,1.2);
  const obstacles = new CameraObstacles();
  obstacles.add([2,1.5,.2],wall);
  const radii = Array(15).fill(.25);
  const group = new GroupCamera([],radii,obstacles);
  const camera = new PerspectiveCamera(45,.65,.1,150);
  const poses = [fighter(-1.5,0),fighter(1.5,0)];
  const focus = new Vector3(0,.8,0), direction = new Vector3();
  for (let frame=0; frame<60; frame++) {
    group.update(camera,poses,false,1/60);
    camera.updateMatrixWorld();
    const distance = camera.position.distanceTo(focus);
    direction.copy(camera.position).sub(focus).normalize();
    expect(obstacles.cast(focus,direction,distance,.15).distance).toBeCloseTo(distance);
    for (const pose of poses) {
      const center = new Vector3().fromArray(pose).project(camera);
      expect(Math.max(Math.abs(center.x), Math.abs(center.y))).toBeLessThan(.7);
    }
    camera.getWorldDirection(direction); direction.y=0; direction.normalize();
    const [x,z] = group.movement(0,-1);
    expect(direction.x).toBeCloseTo(x);
    expect(direction.z).toBeCloseTo(z);
  }
  const pitchBefore = Math.asin(camera.getWorldDirection(direction).y);
  wall.visible = false;
  group.update(camera,poses,false,1/60);
  const first = Math.asin(camera.getWorldDirection(direction).y);
  expect(Math.abs(first-pitchBefore)).toBeLessThan(.07);
  for (let i=0; i<180; i++) group.update(camera,poses,false,1/60);
  expect(Math.asin(camera.getWorldDirection(direction).y)).toBeCloseTo(-Math.atan2(.85,Math.hypot(.18,1)),4);
});

test('a focus inside stacked solids escapes while both fighters remain framed', () => {
  const obstacles = new CameraObstacles();
  for (const y of [1,2.65]) {
    const box = new Object3D();
    box.position.y = y;
    obstacles.add([1,1,1],box);
  }
  const group = new GroupCamera([],Array(15).fill(.25),obstacles);
  const camera = new PerspectiveCamera(45,1,.1,150);
  const poses = [fighter(-2,0),fighter(2,0)];
  group.update(camera,poses,false,1/60);
  camera.updateMatrixWorld();
  const ray = camera.getWorldDirection(new Vector3());
  const t = -camera.position.z / ray.z;
  const focus = camera.position.clone().addScaledVector(ray,t);
  expect(focus.y).toBeGreaterThan(3.65);
  for (const pose of poses) {
    const p = new Vector3().fromArray(pose).project(camera);
    expect(Math.max(Math.abs(p.x),Math.abs(p.y))).toBeLessThan(.7);
  }
});

test('a closing camera stops before an obstruction when the nearer distance still fits the survivor', () => {
  const obstacles = new CameraObstacles();
  const box = new Object3D();
  const direction = new Vector3(.18,.85,1).normalize();
  const focus = new Vector3(0,.8,0);
  box.position.copy(focus).addScaledVector(direction,8);
  obstacles.add([1,1,1],box);
  box.visible = false;
  const group = new GroupCamera([],Array(15).fill(.25),obstacles);
  const camera = new PerspectiveCamera(45,1,.1,150);
  group.update(camera,[fighter(-9,0),fighter(9,0)],false,1/60);
  expect(camera.position.distanceTo(focus)).toBeGreaterThan(15);
  box.visible = true;
  group.update(camera,[fighter(0,0)],false,1/60);
  const limit = obstacles.cast(focus,direction,15,.15).distance;
  expect(camera.position.distanceTo(focus)).toBeCloseTo(limit-.01,4);
  expect(camera.getWorldDirection(new Vector3()).dot(direction)).toBeCloseTo(-1,6);
});

test('an overhead panel permits a shallow view underneath it', () => {
  const obstacles = new CameraObstacles();
  obstacles.fixed([0,3,0],[6,.1,6]);
  const group = new GroupCamera([],Array(15).fill(.25),obstacles);
  const camera = new PerspectiveCamera(45,1,.1,150);
  group.update(camera,[fighter(0,0)],false,1/60);
  expect(camera.position.y).toBeLessThan(2.75);
  expect(camera.position.distanceTo(new Vector3(0,.8,0))).toBeCloseTo(4.5);
});

test('a low obstruction raises the aim without pitching the camera overhead or repeatedly bobbing', () => {
  const vent = new Object3D();
  vent.position.set(0,.55,.9);
  const obstacles = new CameraObstacles();
  obstacles.add([1.25,.55,.7],vent);
  const group = new GroupCamera([],Array(15).fill(.25),obstacles);
  const camera = new PerspectiveCamera(45,1,.1,150);
  const normal = new Vector3(.18,.85,1).normalize();
  const poses = [fighter(-1,0),fighter(1,0)];
  const heights = [];
  for (let frame=0; frame<90; frame++) {
    group.update(camera,poses,false,1/60);
    const ray = camera.getWorldDirection(new Vector3());
    expect(ray.dot(normal)).toBeCloseTo(-1,6);
    heights.push(camera.position.y-camera.position.z*ray.y/ray.z);
  }
  expect(Math.min(...heights)).toBeGreaterThan(1.1);
  expect(Math.max(...heights.slice(-30))-Math.min(...heights.slice(-30))).toBeLessThan(.001);
  vent.visible = false;
  group.update(camera,poses,false,1/60);
  const next = camera.position.y-camera.position.z*.85;
  expect(next).toBeLessThan(heights.at(-1)!);
  expect(next).toBeGreaterThan(.8);
});
