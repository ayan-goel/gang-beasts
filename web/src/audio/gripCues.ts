import { lowStamina, type FighterPhase } from '../game/fighterCondition';
import type { CameraPosition } from './spatialMix';
import type { GripCue, GripKind } from './gripSound';

export interface GripAudioState {
  gripTargets: readonly (readonly [number,number])[];
  stamina: readonly number[];
}

const priority = {strain:3, catch:2, release:1};

export function changedGrip(previous: GripAudioState, current: GripAudioState, phases: readonly FighterPhase[],
    locations: readonly CameraPosition[]): GripCue | undefined {
  let best: GripCue | undefined;
  for (const [slot,hands] of current.gripTargets.entries()) {
    const before = previous.gripTargets[slot], location = locations[slot];
    if (!before || !location || phases[slot] !== 'active') continue;
    let kind: GripKind;
    if (previous.stamina[slot]! > lowStamina && current.stamina[slot]! <= lowStamina &&
        hands.some((id,i) => id !== 0 && id === before[i])) kind = 'strain';
    else if (hands.some((id,i) => id !== 0 && id !== before[i])) kind = 'catch';
    else if (hands.some((id,i) => id === 0 && before[i])) kind = 'release';
    else continue;
    if (!best || priority[kind] > priority[best.kind] ||
        (kind === best.kind && location.distance < best.distance)) {
      best = {kind,pan:location.pan,distance:location.distance};
    }
  }
  return best;
}
