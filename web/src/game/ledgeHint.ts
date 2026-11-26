import type { MatchView } from '../rendering/SkylineView';

export function fixedLedge(objectId: number | undefined, hazard: MatchView['hazard']) {
  // The verified Skyline manifest reserves 1 for the roof and 8/9 for the west/east panels.
  if (objectId === 8) return hazard === 'quiet' || hazard === 'west_warning';
  if (objectId === 9) return hazard !== 'gust_warning' && hazard !== 'gust';
  return objectId === 1;
}

export const pullHint = (key: 'Space' | 'Enter') => 'Hold ' + key + ' to pull up';
