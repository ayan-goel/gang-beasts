import type { MatchView, SkylineMap } from '../rendering/SkylineView';

export function fixedLedge(objectId: number | undefined, hazard: MatchView['hazard'], map?: SkylineMap) {
  if (map?.tour) {
    const kind = map.objects.find(obj => obj.id === objectId)?.kind;
    return kind === 'deck' || kind === 'panel';
  }
  // The verified Skyline manifest reserves 1 for the roof and 8/9 for the west/east panels.
  if (objectId === 8) return hazard === 'quiet' || hazard === 'west_warning';
  if (objectId === 9) return hazard !== 'gust_warning' && hazard !== 'gust';
  return objectId === 1;
}

export const pullHint = (key: 'Space' | 'Enter') => 'Hold ' + key + ' to pull up';
