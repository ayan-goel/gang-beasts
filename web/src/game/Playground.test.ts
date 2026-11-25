import { describe, expect, it } from 'vitest';
import { Playground } from './Playground';

function createFixture() {
  let tick = 0n;
  let pending = false;
  let impulseCount = 0;
  let now = 0;
  let replayEnd: number | undefined;
  const simulation = {
    step() {
      if (replayEnd !== undefined && tick >= BigInt(replayEnd)) return false;
      tick++; if (pending) { impulseCount++; pending = false; }
      return true;
    },
    tick: () => tick,
    reset() { tick = 0n; pending = false; replayEnd = undefined; },
    queue_impulse() { pending = true; },
    fixture_state: () => new Float32Array(14),
    collider_lines: () => new Float32Array(),
    free() {},
    configure(json: string) { if (json === 'invalid') throw new Error('invalid config'); tick = 0n; pending = false; replayEnd = undefined; },
    configuration_json: () => '{}',
    import_recording(json: string) { if (json === 'invalid') throw new Error('invalid recording'); tick = 0n; pending = false; replayEnd = 2; },
    export_recording: () => '{}',
    replay_end: () => replayEnd,
    recording_available: () => true,
  };
  const playground = new Playground(simulation, () => now);
  return {
    playground, simulation, impulseCount: () => impulseCount,
    advance(seconds: number) { now += seconds * 1000; playground.frame(now); },
    elapse(seconds: number) { now += seconds * 1000; },
  };
}

describe('playground playback', () => {
  it('preserves discarded-time evidence through pause and speed changes', () => {
    const { playground, advance } = createFixture();
    advance(30);
    const discarded = playground.snapshot().droppedSeconds;
    expect(discarded).toBeGreaterThan(29);
    playground.setPaused(true);
    playground.setRate(0.5);
    expect(playground.snapshot().droppedSeconds).toBe(discarded);
    playground.reset();
    expect(playground.snapshot().droppedSeconds).toBe(0);
  });

  it('pauses without accumulating debt and single-steps only while paused', () => {
    const { playground, simulation, advance } = createFixture();
    advance(1 / 60);
    expect(simulation.tick()).toBe(1n);
    playground.singleStep();
    expect(simulation.tick()).toBe(1n);
    playground.setPaused(true);
    advance(30);
    expect(simulation.tick()).toBe(1n);
    playground.singleStep();
    expect(simulation.tick()).toBe(2n);
    playground.setPaused(false);
    advance(1 / 60);
    expect(simulation.tick()).toBe(3n);
  });

  it('slow motion changes tick frequency without changing the physics step', () => {
    const { playground, simulation, advance } = createFixture();
    playground.setRate(0.25);
    for (let i = 0; i < 60; i++) advance(1 / 60);
    expect(simulation.tick()).toBe(15n);
  });

  it('keeps one pending impulse and resets all playback debt and commands', () => {
    const { playground, simulation, impulseCount } = createFixture();
    playground.setPaused(true);
    playground.queueImpulse();
    playground.queueImpulse();
    expect(playground.snapshot().impulsePending).toBe(true);
    playground.singleStep();
    expect(impulseCount()).toBe(1);
    expect(playground.snapshot().impulsePending).toBe(false);
    playground.queueImpulse();
    playground.reset();
    expect(simulation.tick()).toBe(0n);
    expect(playground.snapshot().paused).toBe(true);
    expect(playground.snapshot().impulsePending).toBe(false);
    playground.singleStep();
    expect(impulseCount()).toBe(1);
  });

  it('never advances time from before a resume, reset or rate change', () => {
    const { playground, simulation, advance, elapse } = createFixture();
    playground.setPaused(true);
    elapse(30);
    playground.setPaused(false);
    advance(1 / 60);
    expect(simulation.tick()).toBe(1n);
    elapse(30);
    playground.reset();
    advance(1 / 60);
    expect(simulation.tick()).toBe(1n);
    elapse(30);
    playground.setRate(0.5);
    advance(1 / 30);
    expect(simulation.tick()).toBe(2n);
  });

  it('imports paused, stops exactly at the replay end and exits replay on reset', () => {
    const { playground, simulation, advance } = createFixture();
    playground.importRecording('{}');
    advance(1);
    expect(simulation.tick()).toBe(0n);
    playground.setPaused(false);
    advance(1);
    expect(simulation.tick()).toBe(2n);
    expect(playground.snapshot().paused).toBe(true);
    playground.setPaused(false);
    expect(playground.snapshot().paused).toBe(true);
    playground.reset();
    expect(playground.snapshot().replayEnd).toBeUndefined();
  });

  it('a rejected import or tuning change preserves playback and pending input', () => {
    const { playground, simulation } = createFixture();
    playground.queueImpulse();
    expect(() => playground.importRecording('invalid')).toThrow();
    expect(() => playground.configure('invalid')).toThrow();
    expect(playground.snapshot().paused).toBe(false);
    expect(playground.snapshot().impulsePending).toBe(true);
    expect(simulation.tick()).toBe(0n);
  });
});
