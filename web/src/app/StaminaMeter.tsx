import './stamina.css';

export function StaminaMeter({name, points, holding}: {name: string; points: number; holding: boolean}) {
  if (!holding && points >= 100) return null;
  // Keep the endpoints exclusive to actual exhaustion and full recovery.
  const value = points >= 100 ? 100 : Math.min(99, Math.ceil(points));
  return <small className="fighter-stamina" data-low={value <= 25}>
    <span>{value <= 25 ? 'Low stamina' : 'Stamina'}</span>
    <meter aria-label={name + ' stamina'} min={0} max={100} value={value} />
    <span>{value}%</span>
  </small>;
}
