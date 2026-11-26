export const lowStamina = 25;
export const fighterPhases = ['active','knocked_out','recovering','eliminated'] as const;
export type FighterPhase = typeof fighterPhases[number];

export function conditionNotice(name: string, phase: FighterPhase) {
  if (phase === 'knocked_out') return name + ' knocked out';
  if (phase === 'recovering') return name + ' getting up';
  return '';
}
