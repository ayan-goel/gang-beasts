import { expect, test } from 'vitest';
import { MatchCues } from './MatchCues';

function fixture() {
  const cursor = new MatchCues();
  const update = (now: number, targets: [number,number][], stamina = targets.map(() => 100),
      options: {epoch?:string; warning?:boolean; damage?:number} = {}) => cursor.observe(
    options.epoch ?? '1',options.warning ? 'west_warning' : 'quiet',targets.map(() => 'active'),now,
    targets.map((_,i) => ({health:100-(options.damage ?? 0),pan:i ? 0.5 : -0.5,distance:10+i})),
    {gripTargets:targets, stamina});
  return {cursor,update};
}

test('actual hand target changes generate spatial catches and releases, including a retargeted hand', () => {
  const {update} = fixture();
  expect(update(0,[[0,0],[0,0]])).toEqual({});
  expect(update(50,[[0,0],[0,7]]).grip).toEqual({kind:'catch',pan:0.5,distance:11});
  expect(update(100,[[0,0],[0,7]])).toEqual({});
  expect(update(250,[[0,0],[0,9]]).grip?.kind).toBe('catch');
  expect(update(280,[[0,0],[0,0]]).grip?.kind).toBe('release');
});

test('a sustained hold warns once when it crosses the HUD low-stamina threshold', () => {
  const {update} = fixture();
  expect(update(0,[[7,8]],[27])).toEqual({});
  expect(update(50,[[7,8]],[25]).grip?.kind).toBe('strain');
  expect(update(100,[[7,8]],[22])).toEqual({});
  expect(update(150,[[0,0]],[0]).grip?.kind).toBe('release');
  expect(update(200,[[0,0]],[5])).toEqual({});
});

test('a pile shares one grip voice, suppresses chatter and consumes events under higher priority cues', () => {
  const {update} = fixture();
  update(0,[[0,0],[0,0]]);
  expect(update(10,[[7,9],[4,0]]).grip?.pan).toBe(-0.5);
  expect(update(40,[[8,9],[4,0]]).grip).toBeUndefined();
  expect(update(200,[[0,0],[0,0]],undefined,{damage:5}).grip).toBeUndefined();
  expect(update(240,[[0,0],[0,0]],undefined,{damage:5})).toEqual({});
  expect(update(400,[[9,0],[0,0]],undefined,{warning:true})).toEqual({warning:'west_warning'});
});

test('initial holds, stale data, new rounds and reconnects establish a silent baseline', () => {
  const {cursor,update} = fixture();
  expect(update(0,[[2,0]])).toEqual({});
  expect(update(800,[[0,0]])).toEqual({});
  expect(update(900,[[4,0]],undefined,{epoch:'2'})).toEqual({});
  cursor.reset();
  expect(update(910,[[0,0]],undefined,{epoch:'2'})).toEqual({});
});
