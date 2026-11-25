import { LocalRagdoll } from '../generated/wasm/brawler_wasm';
import type { SimulationPort } from './SimulationPort';
import type { RigDefinition, RigInspection } from './rigDefinition';

export class RagdollSimulation implements SimulationPort, RigInspection {
  private readonly native: LocalRagdoll;
  readonly definition: RigDefinition;
  readonly rig = this;
  selectedBody = 2;

  constructor(configuration: string) {
    this.native = new LocalRagdoll(configuration);
    try { this.definition = JSON.parse(this.native.definition_json()) as RigDefinition; }
    catch (error) { this.native.free(); throw error; }
  }

  selectBody(index: number): void {
    if (!Number.isInteger(index) || index < 0 || index >= this.definition.bodies.length) throw new Error('Invalid body selection');
    this.selectedBody = index;
  }

  step(): boolean { this.native.step(); return true; }
  tick(): bigint { return this.native.tick(); }
  reset(): void { this.native.reset(); }
  free(): void { this.native.free(); }
  configure(json: string): void { this.native.configure(json); }
  configuration_json(): string { return this.native.configuration_json(); }
  bodyStates(): Float32Array { return this.native.body_states(); }
  jointStates(): Float32Array { return this.native.joint_states(); }
  collider_lines(): Float32Array { return this.native.collider_lines(); }
  fixture_state(): Float32Array { return this.bodyStates().slice(this.selectedBody * 14, (this.selectedBody + 1) * 14); }
  queue_impulse(x: number, y: number, z: number): void { this.native.queue_impulse(this.selectedBody, x, y, z); }
}
