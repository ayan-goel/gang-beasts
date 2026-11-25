import { MovementInput } from './MovementInput';
import { FixedClock } from './FixedClock';
import type { SimulationPort } from './SimulationPort';
import { inspectRig } from './rigDefinition';

export type PlaybackRate = 0.25 | 0.5 | 1;

export class Playground {
  private clock = new FixedClock();
  private paused = false;
  private rate: PlaybackRate = 1;
  private impulsePending = false;
  private stepMs = 0;
  private previousTime: number;
  private revision = 0;
  readonly movementInput?: MovementInput;
  showColliders = false;
  showJoints = false;

  constructor(readonly simulation: SimulationPort, private readonly now = () => performance.now()) {
    this.previousTime = now();
    const movement = simulation.rig?.movement;
    if (movement) this.movementInput = new MovementInput(movement, () => !this.paused && simulation.rig!.controller!.state()[0] === 1);
  }

  get sessionRevision(): number { return this.revision; }

  frame(timestamp: number): void {
    if (!Number.isFinite(timestamp) || timestamp < this.previousTime) return;
    const seconds = (timestamp - this.previousTime) / 1000;
    this.previousTime = timestamp;
    if (!this.paused) this.step(this.clock.advance(seconds * this.rate));
  }

  setPaused(paused: boolean): void {
    this.paused = paused || this.replayComplete();
    if (this.paused) this.movementInput?.clear();
    // Discard fractional playback debt so resume never consumes pre-pause time.
    this.clock.clearDebt();
    this.previousTime = this.now();
  }

  setRate(rate: PlaybackRate): void {
    if (![0.25, 0.5, 1].includes(rate)) throw new Error('Unsupported playback rate');
    this.rate = rate;
    this.clock.clearDebt();
    this.previousTime = this.now();
  }

  singleStep(): void {
    if (this.paused) this.step(1);
  }

  queueImpulse(impulse: [number, number, number] = [0, 4, 0]): void {
    if (this.impulsePending) return;
    this.simulation.queue_impulse(...impulse);
    this.impulsePending = true;
  }

  reset(): void {
    this.movementInput?.clear();
    this.simulation.reset();
    this.clearPlayback();
  }

  configure(json: string): void {
    this.simulation.configure(json);
    this.movementInput?.clear();
    this.clearPlayback();
    this.paused = true;
  }

  importRecording(json: string): void {
    if (!this.simulation.import_recording) throw new Error('This scene does not support recording import');
    this.simulation.import_recording(json);
    this.clearPlayback();
    this.paused = true;
  }

  exportRecording(): string {
    if (!this.simulation.export_recording) throw new Error('This scene does not support recording export');
    return this.simulation.export_recording();
  }

  private clearPlayback(): void {
    this.revision++;
    this.impulsePending = false;
    this.clock = new FixedClock();
    this.previousTime = this.now();
    this.stepMs = 0;
  }

  setControlEnabled(enabled: boolean): void {
    if (!enabled) this.movementInput?.clear();
    this.simulation.rig?.controller?.setEnabled(enabled);
  }

  snapshot() {
    const state = this.simulation.fixture_state();
    return {
      movementInput: this.movementInput ? { command: this.movementInput.command, run: this.movementInput.run } : undefined,
      tick: this.simulation.tick().toString(), height: state[1]!,
      verticalSpeed: state[8]!, mass: state[13]!,
      paused: this.paused, rate: this.rate, impulsePending: this.impulsePending,
      showColliders: this.showColliders, stepMs: this.stepMs,
      droppedSeconds: this.clock.droppedSeconds,
      debugSegments: this.showColliders ? this.simulation.collider_lines().length / 6 : 0,
      configuration: this.simulation.configuration_json(),
      replayEnd: this.simulation.replay_end?.(), replayComplete: this.replayComplete(),
      recordingAvailable: this.simulation.recording_available?.() ?? false,
      supportsRecording: !!this.simulation.export_recording,
      rig: this.simulation.rig ? inspectRig(this.simulation.rig) : undefined, showJoints: this.showJoints,
    };
  }

  private step(count: number): void {
    if (!count) return;
    const start = performance.now();
    let completed = 0;
    for (let i = 0; i < count; i++) {
      if (!this.simulation.step()) break;
      completed++;
    }
    if (completed) this.stepMs = (performance.now() - start) / completed;
    this.impulsePending = false;
    if (this.replayComplete()) this.setPaused(true);
  }

  private replayComplete(): boolean {
    const end = this.simulation.replay_end?.();
    return end !== undefined && this.simulation.tick() >= BigInt(end);
  }
}

export type PlaygroundSnapshot = ReturnType<Playground['snapshot']>;
