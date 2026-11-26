import type { MatchView } from '../rendering/SkylineView';

export function meleeResult(match: Pick<MatchView, 'phase' | 'winner'>, names: readonly string[]) {
  if (match.winner === null) return 'Round drawn. No point awarded.';
  return `${names[match.winner]} wins the ${match.phase === 'match_results' ? 'match' : 'round'}`;
}
