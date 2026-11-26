import type { LandingEvent } from '../audio/landingSound';
import type { LedgePractice } from './LedgePractice';
import type { ImpactSource } from '../audio/impactSound';
import { arenaGripName, fighterGripName, handGripLabels } from './gripLabels';
import { fixedLedge } from './ledgeHint';
import type { FighterPhase } from './fighterCondition';
import type { Object3D } from 'three';
import type { CameraPosition } from '../audio/spatialMix';
import type { SkylineContent } from '../lobby/content';
import type { MatchView } from '../rendering/SkylineView';
import { LocalDuel } from '../generated/wasm/brawler_wasm';
import configuration from '../generated/physics.json?raw';
import { FixtureScene } from '../rendering/FixtureScene';
import { bindMovementInput } from './bindMovementInput';
import { bindCameraOrbit } from './bindCameraOrbit';
import { FixedClock } from './FixedClock';
import { MovementInput } from './MovementInput';
import type { RigDefinition } from './rigDefinition';

type Player = 0 | 1;
export interface DuelHit {
  attacker: Player; defender: Player; hand: Player | null; action: 'punch' | 'headbutt'; region: string;
  damage: number; attackId: string; tick: string;
}
export interface FighterCondition {
  phase: FighterPhase;
  footstep?: number | null;
  landing?: LandingEvent | null;
  health: number; knockouts: number; sinceTick: string;
}
export interface DuelState {
  match?: MatchView;
  map?: SkylineContent['map'];
  impacts: ImpactSource[];
  breakaway?: CameraPosition;
  conditions: [FighterCondition, FighterCondition];
  tick: string;
  paused: boolean;
  hits: [number, number];
  grips: [string[], string[]];
  gripTargets: [[number,number],[number,number]];
  fixedLedge: [boolean, boolean];
  stamina: [number, number];
  latest?: DuelHit;
}

export function startDuel(container: HTMLElement, publish: (snapshot: DuelState) => void, model?: Object3D, practice?: LedgePractice, series?: {seed:number; wins:number}) {
  const skyline = model !== undefined;
  const duel = practice?.duel ?? (series ? LocalDuel.tour_match(configuration, series.seed, series.wins) : skyline ? LocalDuel.skyline_match(configuration) : new LocalDuel(configuration));
  const clock = new FixedClock();
  let frame = 0;
  let scene: FixtureScene | undefined;
  let unbindOrbit: (() => void) | undefined;
  let unbind: (() => void) | undefined;
  let paused = true;
  let hits: [number, number] = [0, 0];
  let latest: DuelHit | undefined;
  let previous = performance.now();
  let displayed = 0;
  let disposed = false;
  const resetClock = () => { clock.clearDebt(); previous = performance.now(); };
  const dispose = () => {
    if (disposed) return;
    disposed = true;
    cancelAnimationFrame(frame);
    document.removeEventListener('visibilitychange', resetClock);
    unbindOrbit?.();
    try { unbind?.(); } finally { try { scene?.dispose(); } finally { duel.free(); } }
  };
  try {
    const definition: RigDefinition = JSON.parse(duel.definition_json());
    let map: SkylineContent['map'] | undefined = skyline ? JSON.parse(duel.map_json()!) : undefined;
    scene = new FixtureScene(container, definition, { map, model });
    scene.addOpponent(definition);
    const renderer = scene;
    const canvas = container.querySelector('canvas')!;
    let match: MatchView | undefined = skyline ? JSON.parse(duel.match_json()!) : undefined;
    const orbit = renderer.groupCamera;
    const move = (id: Player, x: number, z: number, run: boolean) => {
      if (orbit) [x,z] = orbit.movement(x,z);
      duel.set_input(id,x,z,run);
    };
    if (orbit) unbindOrbit = bindCameraOrbit(canvas, orbit, () => !paused);
    const playerInput = (id: Player) => new MovementInput({
      setInput: (x, z, run) => move(id, x, z, run),
      setHeadbutt: held => duel.set_headbutt(id, held),
      clearHeadbutt: () => duel.clear_headbutt(id),
      setLift: held => duel.set_lift(id, held),
      setJump: held => duel.set_jump(id, held),
      clearJumpInput: () => duel.clear_jump(id),
      setHand: (side, held) => duel.set_hand(id, side, held),
      clearHands: () => duel.clear_hands(id),
    }, () => !paused && (!match || match.phase === 'playing'));
    const inputs = [playerInput(0), playerInput(1)];
    unbind = bindMovementInput(canvas, inputs[0]!, practice ? undefined : inputs[1]!);
    const readGrips = (owner: Player) => {
      let onLedge = false;
      const targets: [number,number] = [0,0];
      const labels = handGripLabels(side => {
        const region = duel.grip_region(owner, side);
        if (region !== undefined) {
          targets[side] = 1 + (1 - owner) * 16 + region;
          return fighterGripName(owner === 0 ? 'Blue' : 'Coral', definition.bodies[region]!.part);
        }
        if (!map || !match) return undefined;
        const target = duel.arena_grip(owner, side);
        if (target !== undefined) targets[side] = 65536 + target;
        onLedge ||= fixedLedge(target, match.hazard, map);
        return arenaGripName(map, target);
      });
      return {labels, onLedge, targets};
    };
    const snapshot = () => {
      const first = readGrips(0), second = readGrips(1);
      const conditions: [FighterCondition, FighterCondition] = [JSON.parse(duel.condition_json(0)), JSON.parse(duel.condition_json(1))];
      publish({
        match, map, tick: duel.tick().toString(), hits: [...hits], latest, paused,
        gripTargets: [first.targets, second.targets],
        grips: [first.labels, second.labels], fixedLedge: [first.onLedge, second.onLedge],
        stamina: [duel.stamina(0), duel.stamina(1)],
        breakaway: map && !map.tour ? renderer.breakawaySource(map, Number(duel.tick())) : undefined,
        conditions, impacts: renderer.impactSources(conditions, Number(duel.tick())),
      });
    };
    const render = (dt = 0) => {
      if (renderer.graphicsInterrupted) return;
      const first = duel.body_states(0), second = duel.body_states(1);
      if (match) {
        practice?.observe(first, match);
        const turned = renderer.renderSkyline([first, second], duel.arena_poses(), match, dt, undefined, practice ? 0 : undefined);
        if (turned) for (const id of [0,1] as const) {
          const [x,z,run] = inputs[id]!.command;
          if (x || z) move(id,x,z,run);
        }
      }
      else renderer.renderDuel(first, second);
    };
    const step = () => {
      duel.step();
      // Read every completed tick, including each catch-up step, before a newer
      // native event list can replace a hit that the renderer has not displayed.
      const records = duel.hits_json();
      if (records) for (const hit of JSON.parse(records) as DuelHit[]) {
        hits[hit.attacker]++;
        latest = hit;
      }
    };
    renderer.onGraphicsChange = lost => {
      if (lost) inputs.forEach(input => input.clear());
      resetClock();
      snapshot();
    };
    const animate = (now: number) => {
      if (renderer.graphicsInterrupted) {
        frame = requestAnimationFrame(animate);
        return;
      }
      const previousHit = latest;
      if (!paused && (!skyline || !document.hidden)) {
        const waitingTicks = match?.phase === 'countdown' || match?.phase === 'round_results' ? match.remaining : 0;
        const steps = clock.advance((now - previous) / 1000, waitingTicks);
        for (let i = 0; i < steps; i++) step();
      }
      if (skyline && !paused) {
        const next: MatchView = JSON.parse(duel.match_json()!);
        if (match?.phase === 'playing' && next.phase !== 'playing') inputs.forEach(input => input.clear());
        if (next.epoch !== match?.epoch) {
          map = JSON.parse(duel.map_json()!);
          renderer.setArena(map!);
        }
        match = next;
      }
      render((now - previous) / 1000);
      if (practice?.finished && !paused) {
        paused = true;
        inputs.forEach(input => input.clear());
      }
      previous = now;
      if (now - displayed >= 100 || latest !== previousHit) { snapshot(); displayed = now; }
      frame = requestAnimationFrame(animate);
    };
    render();
    snapshot();
    if (skyline) document.addEventListener('visibilitychange', resetClock);
    frame = requestAnimationFrame(animate);
    return {
      toggle() {
        paused = !paused;
        if (paused) inputs.forEach(input => input.clear());
        else canvas.focus({ preventScroll: true });
        resetClock();
        snapshot();
      },
      reset() {
        paused = true;
        inputs.forEach(input => input.clear());
        duel.reset();
        if (skyline) match = JSON.parse(duel.match_json()!);
        hits = [0, 0];
        latest = undefined;
        clock.clearDebt();
        render();
        snapshot();
      },
      dispose,
    };
  } catch (error) { dispose(); throw error; }
}
