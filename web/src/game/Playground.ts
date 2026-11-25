import { FixedClock } from './FixedClock';

export interface SimulationPort {
  step(): void;
  tick(): bigint;
  reset(): void;
  queue_impulse(x: number, y: number, z: number): void;
  fixture_state(): Float32Array;
  collider_lines(): Float32Array;
  free(): void;
}

export type PlaybackRate = 0.25 | 0.5 | 1;

export class Playground {
  private clock = new FixedClock();
  private paused = false;
  private rate: PlaybackRate = 1;
  private impulsePending = false;
  private stepMs = 0;
  private previousTime: number;
  showColliders = false;

  constructor(readonly simulation: SimulationPort, private readonly now = () => performance.now()) {
    this.previousTime = now();
  }

  frame(timestamp: number): void {
    if (!Number.isFinite(timestamp) || timestamp < this.previousTime) return;
    const seconds = (timestamp - this.previousTime) / 1000;
    this.previousTime = timestamp;
    if (!this.paused) this.step(this.clock.advance(seconds * this.rate));
  }

  setPaused(paused: boolean): void {
    this.paused = paused;
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

  queueImpulse(): void {
    if (this.impulsePending) return;
    this.simulation.queue_impulse(0, 4, 0);
    this.impulsePending = true;
  }

  reset(): void {
    this.simulation.reset();
    this.impulsePending = false;
    this.clock = new FixedClock();
    this.previousTime = this.now();
    this.stepMs = 0;
  }

  snapshot() {
    const state = this.simulation.fixture_state();
    return {
      tick: this.simulation.tick().toString(), height: state[1]!,
      verticalSpeed: state[8]!, mass: state[13]!,
      paused: this.paused, rate: this.rate, impulsePending: this.impulsePending,
      showColliders: this.showColliders, stepMs: this.stepMs,
      droppedSeconds: this.clock.droppedSeconds,
      debugSegments: this.showColliders ? this.simulation.collider_lines().length / 6 : 0,
    };
  }

  private step(count: number): void {
    if (!count) return;
    const start = performance.now();
    for (let i = 0; i < count; i++) this.simulation.step();
    this.stepMs = (performance.now() - start) / count;
    this.impulsePending = false;
  }
}

export type PlaygroundSnapshot = ReturnType<Playground['snapshot']>;
