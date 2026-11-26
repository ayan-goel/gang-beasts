export function GripNotices({names, grips, hints}: {names: readonly string[]; grips: readonly string[][]; hints?: readonly string[]}) {
  return grips.map((hands, slot) => hands.length ? <div key={slot} aria-label={names[slot] + ' grips'}>
    <b>{names[slot]}</b> {hands.join(' · ')}{hints?.[slot] && ' · ' + hints[slot]}
  </div> : null);
}
