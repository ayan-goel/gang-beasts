import type { MatchView } from '../rendering/SkylineView';

type CameraMatch = Pick<MatchView, 'epoch' | 'phase' | 'visible'>;
export interface CameraChoice { epoch: string; slot: number }

export function canSpectate(match: CameraMatch, own: number | null) {
  return (match.phase === 'playing' || match.phase === 'round_results') &&
    (own === null || !match.visible[own]);
}

export function nextSurvivor(visible: boolean[], current: number | null, direction: -1 | 1) {
  let slot = current ?? (direction === 1 ? -1 : 0);
  for (let step = 0; step < visible.length; step++) {
    slot = (slot + direction + visible.length) % visible.length;
    if (visible[slot]) return slot;
  }
  return null;
}

export function followedFighter(match: CameraMatch, own: number | null, choice: CameraChoice | null) {
  // A roster change can reuse the same slot for someone else. A new round always opens in group view.
  if (!choice || choice.epoch !== match.epoch || !canSpectate(match, own)) return null;
  return match.visible[choice.slot] ? choice.slot : nextSurvivor(match.visible, choice.slot, 1);
}
