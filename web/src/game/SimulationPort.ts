import type { RigInspection } from './rigDefinition';

export interface SimulationPort {
  step(): boolean;
  tick(): bigint;
  reset(): void;
  queue_impulse(x: number, y: number, z: number): void;
  fixture_state(): Float32Array;
  collider_lines(): Float32Array;
  free(): void;
  configure(json: string): void;
  configuration_json(): string;
  import_recording?(json: string): void;
  export_recording?(): string;
  replay_end?(): number | undefined;
  recording_available?(): boolean;
  readonly rig?: RigInspection;
}
