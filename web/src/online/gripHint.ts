import { groupOf } from './SnapshotTimeline';

export function gripHint(slot: number, grips: readonly [number, number][]) {
  const hands = grips[slot]!;
  const holdingOpponent = hands.some(id => id > 0 && groupOf(id) < grips.length);
  const held = grips.some(pair => pair.some(id => id > 0 && groupOf(id) === slot));
  if (holdingOpponent && held) return 'Releasing your hands leaves the other fighter’s grip attached.';
  if (holdingOpponent) return 'Hold E to lift. Release your gripping hands to throw.';
  if (!held) return '';
  return hands.includes(0) ? 'Hold J or K to grab back with a free hand.' : 'Free a hand to grab back.';
}
