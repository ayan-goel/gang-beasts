export function GripNotices({names, grips}: {names: readonly string[]; grips: readonly string[][]}) {
  return grips.map((hands, slot) => hands.length ? <div key={slot} aria-label={names[slot] + ' grips'}>
    <b>{names[slot]}</b> {hands.join(' · ')}
  </div> : null);
}
