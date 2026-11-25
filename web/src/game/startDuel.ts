import { LocalDuel } from '../generated/wasm/brawler_wasm';
import configuration from '../generated/physics.json?raw';
import { FixtureScene } from '../rendering/FixtureScene';
import { bindMovementInput } from './bindMovementInput';
import { FixedClock } from './FixedClock';
import { MovementInput } from './MovementInput';
import type { RigDefinition } from './rigDefinition';

type Player = 0 | 1;
export interface DuelHit {
  attacker: Player; defender: Player; hand: Player; region: string;
  damage: number; attackId: string; tick: string;
}
export interface FighterCondition {
  phase: 'active' | 'knocked_out' | 'recovering';
  health: number; knockouts: number; sinceTick: string;
}
export interface DuelState {
  conditions: [FighterCondition, FighterCondition];
  tick: string;
  paused: boolean;
  hits: [number, number];
  grips: [string[], string[]];
  stamina: [number, number];
  latest?: DuelHit;
}

export function startDuel(container: HTMLElement, publish: (snapshot: DuelState) => void) {
  const duel = new LocalDuel(configuration);
  const clock = new FixedClock();
  let frame = 0;
  let scene: FixtureScene | undefined;
  let unbind: (() => void) | undefined;
  let paused = true;
  let hits: [number, number] = [0, 0];
  let latest: DuelHit | undefined;
  let previous = performance.now();
  let displayed = 0;
  let disposed = false;
  const dispose = () => {
    if (disposed) return;
    disposed = true;
    cancelAnimationFrame(frame);
    try { unbind?.(); } finally { try { scene?.dispose(); } finally { duel.free(); } }
  };
  try {
    const definition: RigDefinition = JSON.parse(duel.definition_json());
    scene = new FixtureScene(container, definition);
    scene.addOpponent(definition);
    const renderer = scene;
    const canvas = container.querySelector('canvas')!;
    const playerInput = (id: Player) => new MovementInput({
      setInput: (x, z, run) => duel.set_input(id, x, z, run),
      setLift: held => duel.set_lift(id, held),
      setJump: held => duel.set_jump(id, held),
      clearJumpInput: () => duel.clear_jump(id),
      setHand: (side, held) => duel.set_hand(id, side, held),
      clearHands: () => duel.clear_hands(id),
    }, () => !paused);
    const inputs = [playerInput(0), playerInput(1)];
    unbind = bindMovementInput(canvas, inputs[0]!, inputs[1]!);
    const readGrips = (owner: Player) => ([0, 1] as const).flatMap(side => {
      const region = duel.grip_region(owner, side);
      if (region === undefined) return [];
      const part = definition.bodies[region]!.part.replaceAll('_', ' ');
      return [`${side === 0 ? 'Left' : 'Right'} hand on ${owner === 0 ? 'Blue' : 'Coral'}’s ${part}`];
    });
    const snapshot = () => publish({
      tick: duel.tick().toString(), hits: [...hits], latest, paused, grips: [readGrips(0), readGrips(1)],
      stamina: [duel.stamina(0), duel.stamina(1)],
      conditions: [JSON.parse(duel.condition_json(0)), JSON.parse(duel.condition_json(1))],
    });
    const render = () => renderer.renderDuel(duel.body_states(0), duel.body_states(1));
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
    const animate = (now: number) => {
      if (!paused) {
        const steps = clock.advance((now - previous) / 1000);
        for (let i = 0; i < steps; i++) step();
      }
      previous = now;
      render();
      if (now - displayed >= 100) { snapshot(); displayed = now; }
      frame = requestAnimationFrame(animate);
    };
    render();
    snapshot();
    frame = requestAnimationFrame(animate);
    return {
      toggle() {
        paused = !paused;
        if (paused) inputs.forEach(input => input.clear());
        else canvas.focus({ preventScroll: true });
        clock.clearDebt();
        previous = performance.now();
        snapshot();
      },
      reset() {
        paused = true;
        inputs.forEach(input => input.clear());
        duel.reset();
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
