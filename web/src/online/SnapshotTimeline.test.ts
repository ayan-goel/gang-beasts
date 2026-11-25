import { expect, it } from 'vitest';
import type { BodyPose, Snapshot } from '../net/snapshot';
import { SnapshotTimeline } from './SnapshotTimeline';
import { writePose } from './samplePose';

const dt = 1000/60;
function frame(tick: number, x: number, round = 1n, rotation: BodyPose['rotation'] = [0,0,0,1]): Snapshot {
  return {tick:tick>>>0, round, applied:null, number:Number(round), arenaTick:tick>>>0, phase:1, winner:null,
    remaining:0, fighters:[], bodies:new Map([[1,{position:[x,0,0],rotation}]])};
}
function pose(timeline: SnapshotTimeline, time: number, id=1) {
  const sample = timeline.sample(time)!;
  const values = new Float32Array(7);
  return writePose(sample,id,values,0) ? values : null;
}

it('interpolates a shared tick and takes the short rotational arc across the sign boundary', () => {
  const t = new SnapshotTimeline();
  for(let i=0; i<=10; i+=2) t.push(frame(i,i/10,1n,i===6 ? [0,-Math.SQRT1_2,0,-Math.SQRT1_2] : [0,0,0,1]),i*dt);
  const p=pose(t,10*dt)!;
  expect(p[0]).toBeCloseTo(0.5);
  expect(Math.abs(p[4]!)).toBeCloseTo(Math.sin(Math.PI/8));
  expect(Math.hypot(...p.slice(3))).toBeCloseTo(1);
});

it('applies spawns, removals and teleports at their exact boundary and clears history on a new round', () => {
  const t = new SnapshotTimeline();
  const a=frame(0,0), b=frame(2,20);
  a.bodies.set(2,a.bodies.get(1)!);
  b.bodies.set(2,{position:[.2,0,0],rotation:[0,0,0,1]});
  a.bodies.set(65537,a.bodies.get(1)!);
  b.bodies.set(65538,b.bodies.get(1)!);
  t.push(a,0); t.push(b,2*dt);
  expect(pose(t,6*dt)![0]).toBe(0);
  expect(pose(t,6*dt,2)![0]).toBe(0);
  expect(pose(t,6*dt,65537)).not.toBeNull();
  expect(pose(t,6*dt,65538)).toBeNull();
  expect(pose(t,7*dt,65537)).toBeNull();
  expect(pose(t,7*dt)![0]).toBe(20);
  t.push(frame(4,-8,2n),8*dt);
  expect(t.push(frame(6,99,1n),9*dt)).toBe(false);
  expect(pose(t,9*dt)![0]).toBe(-8);
});

it('extrapolates briefly after loss then freezes, with bounded history, delay and monotonic render time', () => {
  const t = new SnapshotTimeline();
  for(let i=0; i<=60; i+=2) t.push(frame(i,i/10),i*dt);
  expect(t.size).toBeLessThanOrEqual(32);
  expect(t.sample(68*dt)!.status).toBe('estimating');
  const stopped=pose(t,200*dt)![0];
  expect(stopped).toBeCloseTo(6.6);
  expect(pose(t,400*dt)![0]).toBe(stopped);
  expect(t.sample(400*dt)!.status).toBe('stalled');
  const resume=frame(440,9);
  t.push(resume,440*dt);
  expect(pose(t,441*dt)![0]).toBeCloseTo(9);
  for(let i=442; i<=500; i+=2) t.push(frame(i,9), (i + (i%4 ? .9 : 0))*dt);
  expect(t.delayTicks).toBeGreaterThanOrEqual(4);
  expect(t.delayTicks).toBeLessThanOrEqual(6);
});

it('unwraps ticks and rejects duplicate, backward and half-range observations', () => {
  const t = new SnapshotTimeline();
  t.push(frame(0xfffffffe,0),0);
  expect(t.push(frame(0,0.2),2*dt)).toBe(true);
  expect(t.push(frame(0,4),3*dt)).toBe(false);
  expect(t.push(frame(0xffffffff,4),4*dt)).toBe(false);
  expect(t.push(frame(0x80000000,4),5*dt)).toBe(false);
  expect(pose(t,6*dt)![0]).toBeCloseTo(.1);
});

it('keeps presentation time monotonic when a delayed record arrives and adapts its delay within the budget', () => {
  const t = new SnapshotTimeline();
  for (let i=0; i<=10; i+=2) t.push(frame(i,i/10),i*dt);
  const previous = pose(t,19*dt)![0]!;
  t.push(frame(12,1.2),19*dt);
  expect(pose(t,19*dt)![0]).toBeCloseTo(previous);
  for(let i=14;i<180;i+=2) t.push(frame(i,i/10),(i+7+(i%4 ? 1.9 : 0))*dt);
  expect(t.delayTicks).toBeCloseTo(6);
  expect(t.size).toBe(32);
  for(let i=180;i<=400;i+=2) t.push(frame(i,i/10),(i+7)*dt);
  expect(t.delayTicks).toBeCloseTo(4);
});
