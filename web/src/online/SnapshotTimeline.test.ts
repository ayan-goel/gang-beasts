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
  expect(t.delayTicks).toBe(5);
  for(let i=442; i<=500; i+=2) t.push(frame(i,9), (i + (i%4 ? .9 : 0))*dt);
  expect(t.delayTicks).toBeGreaterThanOrEqual(3.4);
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
  for(let i=180;i<320;i+=2) t.push(frame(i,i/10),(i+7+(i%4 ? 1.9 : 0))*dt);
  expect(t.delayTicks).toBeCloseTo(4.4);
  expect(t.size).toBe(32);
  for(let i=320;i<=560;i+=2) t.push(frame(i,i/10),(i+7)*dt);
  expect(t.delayTicks).toBeCloseTo(2.5);
});

it('presents smooth authoritative motion with a buffer sized to regular regional delivery', () => {
  const timeline = new SnapshotTimeline();
  const variation = [0, .3, -.3, .15];
  let index = 0;
  let last: number | undefined;
  for (let now = 0; now <= 200; now += .5) {
    while (index * 2 + 2 + variation[index % 4]! <= now) {
      const tick = index * 2;
      timeline.push(frame(tick,tick / 100), (tick + 2 + variation[index % 4]!) * dt);
      index++;
    }
    if (now < 120) continue;
    const sample = timeline.sample(now * dt)!;
    expect(sample.status).toBe('live');
    const x = pose(timeline,now * dt)![0]! * 100;
    // At 30 Hz with 10 ms of arrival spread, all bodies can stay bracketed within 3.5 ticks of the fastest arrival.
    expect(now - 1.7 - x).toBeLessThanOrEqual(3.5);
    if (last !== undefined) expect(x - last).toBeCloseTo(.5,3);
    last = x;
  }
});

it('allows for skipped publications even if the packets that arrive are exactly on time', () => {
  const timeline = new SnapshotTimeline();
  for (let tick=0; tick<200; tick+=2) {
    if (tick % 12 === 8 || tick % 12 === 10) continue;
    timeline.push(frame(tick,tick/100),tick*dt);
    if (tick > 120 && tick % 12 === 6) {
      expect(timeline.sample((tick+5.9)*dt)!.status).toBe('live');
    }
  }
});

it('caps buffering on repeatedly delayed publications and starts a new clock after a silence', () => {
  const timeline = new SnapshotTimeline();
  for (let tick = 0; tick <= 240; tick += 8) timeline.push(frame(tick, 0), tick * dt);
  expect(timeline.delayTicks).toBe(6);
  const frozen = timeline.sample(270 * dt)!;
  expect(frozen.status).toBe('stalled');
  const received = 285 * dt;
  timeline.push(frame(272, .5), received);
  expect(timeline.size).toBe(1);
  expect(timeline.delayTicks).toBe(5);
  expect(pose(timeline, received)![0]).toBeCloseTo(.5);
});

it('retains enough history while the shared clock adjusts to a slower route', () => {
  const timeline = new SnapshotTimeline();
  for (let tick = 0; tick <= 300; tick += 2) {
    const arrival = tick + (tick >= 160 ? 4 : 0);
    timeline.push(frame(tick,tick / 100), arrival * dt);
    // The slower path is now steady, but its clock origin still takes time to converge.
    if (tick >= 250) expect(timeline.sample((arrival + 1.95) * dt)!.status).toBe('live');
  }
});

it('catches up to fresh poses and round state after queued snapshots arrive together', () => {
  const timeline = new SnapshotTimeline();
  timeline.push(frame(0, 0), 0);
  timeline.sample(0);
  const delayedUntil = 110 * dt;
  // A busy client receives the first post-stall record late, then drains the socket in order.
  for (let tick = 2; tick <= 110; tick += 2) {
    const state = frame(tick, tick / 100);
    state.phase = tick < 90 ? 0 : 1;
    timeline.push(state, delayedUntil + tick / 100);
  }
  const now = delayedUntil + 1.1;
  const caughtUp = timeline.sample(now)!;
  expect(caughtUp.status).toBe('live');
  expect(caughtUp.state.phase).toBe(1);
  const presented = pose(timeline, now)![0]! * 100;
  expect(110 - presented).toBeGreaterThanOrEqual(2);
  expect(110 - presented).toBeLessThanOrEqual(timeline.delayTicks + 2.01);
  let previous = presented;
  for (let tick = 112; tick <= 132; tick += 2) {
    const at = tick * dt + 1.1;
    timeline.push(frame(tick, tick / 100), at);
    expect(timeline.sample(at)!.status).toBe('live');
    const next = pose(timeline, at)![0]! * 100;
    expect(next).toBeGreaterThan(previous);
    previous = next;
  }
});
