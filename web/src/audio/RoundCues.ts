import type { MatchView } from '../rendering/SkylineView';

type RoundState = Pick<MatchView, 'epoch' | 'phase' | 'remaining' | 'winner'>;
export type RoundCue = 'count' | 'go' | 'round_win' | 'draw' | 'match_win';
const beat = (ticks: number) => Math.ceil(ticks / 60);

export class RoundCues {
  private previous?: RoundState;
  private observedAt = 0;

  reset() { this.previous = undefined; }

  observe(match: RoundState, now: number): RoundCue | undefined {
    const previous = this.previous;
    const gap = now - this.observedAt;
    this.previous = match;
    this.observedAt = now;
    // The first visible state after a stall establishes the clock without replaying missed beats.
    if (!previous || gap > 500) return;
    if (match.epoch !== previous.epoch) {
      if (previous.phase === 'round_results' && match.phase === 'countdown' && beat(match.remaining) === 3) return 'count';
      return;
    }
    if (previous.phase === 'countdown') {
      if (match.phase === 'playing') return 'go';
      if (match.phase === 'countdown' && beat(match.remaining) < beat(previous.remaining)) return 'count';
    }
    if (previous.phase === 'playing' && match.phase === 'round_results') return match.winner === null ? 'draw' : 'round_win';
    if (previous.phase === 'round_results' && match.phase === 'match_results') return 'match_win';
  }
}
