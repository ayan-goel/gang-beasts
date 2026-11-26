import { LocalDuel } from '../generated/wasm/brawler_wasm';
import configuration from '../generated/physics.json?raw';
import type { MatchView } from '../rendering/SkylineView';

export class LedgePractice {
  readonly duel = LocalDuel.skyline_match(configuration);
  status: 'ready' | 'hanging' | 'falling' | 'landing' | 'recovered' | 'missed' = 'ready';
  private caught = false;
  private standingSince?: bigint;

  get finished() { return this.status === 'recovered' || this.status === 'missed'; }

  observe(pose: Float32Array, match: MatchView) {
    if (this.finished) return;
    const held = [0, 1].some(hand => this.duel.arena_grip(0, hand) === 1);
    const x = pose[28]!, y = pose[29]!;
    this.caught ||= held && x < -10 && y < 0;
    if (match.phase !== 'playing' || !match.visible[0]) this.status = 'missed';
    else if (this.caught && !held && x > -10 && y > 0.55) {
      // A climber can momentarily crest the rim and roll back out. Count a settled second.
      const tick = this.duel.tick();
      this.standingSince ??= tick;
      this.status = tick - this.standingSince >= 60n ? 'recovered' : 'landing';
    } else {
      this.standingSince = undefined;
      this.status = held && this.caught ? 'hanging' : y < 0 ? 'falling' : this.caught ? 'landing' : 'ready';
    }
  }
}

export async function prepareLedgePractice(signal: AbortSignal) {
  signal.throwIfAborted();
  const practice = new LedgePractice();
  try {
    // Replay the real approach so the panel, limbs and contacts have their match velocities.
    // Yield between batches so the player can leave the scene while it is preparing.
    const countdown = 180;
    let tick = -countdown;
    while (tick < 7150) {
      signal.throwIfAborted();
      const deadline = performance.now() + 8;
      do {
        if (tick === 6800) practice.duel.set_input(0, -1, 0, false);
        if (tick === 7050) practice.duel.set_input(0, 1, 0, false);
        if (tick === 7078) practice.duel.set_input(0, 0, 0, false);
        practice.duel.step();
        tick++;
      } while (tick < 7150 && performance.now() < deadline);
      if (tick < 7150) await new Promise(resolve => setTimeout(resolve, 0));
    }
    return practice;
  } catch (error) {
    practice.duel.free();
    throw error;
  }
}
