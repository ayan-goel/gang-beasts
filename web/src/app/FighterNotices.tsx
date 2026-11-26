import { conditionNotice, type FighterPhase } from '../game/fighterCondition';

export function FighterNotices({names, phases}: {names: readonly string[]; phases: readonly FighterPhase[]}) {
  return phases.map((phase, slot) => {
    const notice = conditionNotice(names[slot]!, phase);
    return notice ? <span key={slot}>{notice}</span> : null;
  });
}
