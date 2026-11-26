import { buttons, quantizeDirection } from '../../src/net/inputFrame';
import type { FighterState } from '../../src/net/snapshot';

export function chooseInput(match: number, round: number, tick: number, slot: number, x: number, z: number) {
  // Every third match includes a fight through both collapses and the gust.
  const fightUntil = match % 3 === 1 && round === 1 ? 10920 : 360;
  if (tick >= fightUntil) {
    const fleeing = slot !== match % 8;
    return {axes:quantizeDirection(fleeing ? 1 : 0,0), held:fleeing ? buttons.run : 0};
  }
  const pulse = Math.floor(tick / 6);
  const phase = (pulse + slot * 3) % 24;
  let held = phase < 14 ? buttons.leftHand : 0;
  if (phase === 2 || phase === 18) held |= buttons.rightHand;
  if (phase >= 8 && phase < 14) held |= buttons.lift;
  if ((round + match) % 3 === 0) {
    // Seeded variation is reproducible for a given fighter and input interval.
    const bits = Math.imul((pulse + 1) ^ ((slot + 1) << 16) ^ round,0x9e3779b1) >>> 0;
    if (bits % 13 === 0) held |= buttons.jump;
    if (bits % 11 === 0) held |= buttons.headbutt;
  }
  const length = Math.max(.5, Math.hypot(x,z));
  return {axes:quantizeDirection(-x/length,-z/length),held};
}

export function verifyGrips(fighters: readonly FighterState[]) {
  let count = 0;
  for (const owner of fighters) {
    for (const target of owner.grips) {
      if (!target) continue;
      if (owner.consciousness === 3 || (target < 65536 && fighters[Math.floor((target-1)/16)]?.consciousness === 3)) {
        throw new Error('Grip attached to an eliminated fighter');
      }
      count++;
    }
  }
  return count;
}
