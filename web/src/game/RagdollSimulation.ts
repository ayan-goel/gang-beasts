import { LocalRagdoll } from '../generated/wasm/brawler_wasm';
import type { SimulationPort } from './SimulationPort';
import type { RigDefinition, RigInspection } from './rigDefinition';
import type { MotorConfiguration, RigControl } from './motorInspection';

export class RagdollSimulation implements SimulationPort, RigInspection {
  private readonly native: LocalRagdoll;
  readonly definition: RigDefinition;
  readonly rig = this;
  readonly controller?: RigControl;
  selectedBody = 2;

  constructor(configuration: string, standing = false) {
    this.native = standing ? LocalRagdoll.standing(configuration) : new LocalRagdoll(configuration);
    try {
      this.definition = JSON.parse(this.native.definition_json()) as RigDefinition;
      const motorJson = this.native.motor_configuration_json();
      if (motorJson) this.controller = {
        configuration: JSON.parse(motorJson) as MotorConfiguration,
        state: () => this.native.controller_state(), efforts: () => this.native.motor_efforts(),
        targets: () => this.native.motor_targets(), setEnabled: enabled => this.native.set_enabled(enabled),
      };
    }
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
