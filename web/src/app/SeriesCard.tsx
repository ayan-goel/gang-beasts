import type { MatchView, SkylineMap } from '../rendering/SkylineView';
import './series.css';

export function arenaName(map?: SkylineMap) {
  return map?.tour?.name ?? 'Skyline';
}

export function stageWarning(map: SkylineMap | undefined, match: MatchView, skylineWarning: string) {
  if (!map?.tour) return skylineWarning;
  const tick = match.arenaTick ?? 0;
  // The same deadlines move the colliders on the host. Give players three seconds
  // to leave the affected surface, then clear the callout so it cannot mask play.
  const motion = map.tour.motions.find(m => m.start - 180 <= tick && tick <= m.start + 45);
  if (!motion) return '';
  const prop = map.objects.find(o => o.id === motion.object)?.art?.replaceAll('-', ' ') ?? 'Platform';
  if (prop === 'shark') return 'Shark breach · Clear the rail!';
  const action = {slide:'moving', heave:'surging', sweep:'sweeping', roll:'tilting', sink:'shifting'}[motion.kind];
  return prop[0]!.toUpperCase() + prop.slice(1) + ' ' + action;
}

export function SeriesCard({match, map, names, wins}: {
  match: MatchView; map?: SkylineMap; names: readonly string[]; wins: number;
}) {
  if (match.phase === 'playing') return null;
  if (match.phase === 'countdown') return <div className="tour-intro" role="status">
    <small>Round {match.round} · First to {wins}</small>
    <h2>{arenaName(map)}</h2>
    <p>{map?.tour?.twist ?? 'The service walkways break away before the crosswind hits.'}</p>
    <b aria-label="Countdown">{Math.max(1,Math.ceil(match.remaining/60))}</b>
  </div>;
  const complete = match.phase === 'match_results';
  const winner = match.winner === null ? null : names[match.winner];
  const heading = winner === null ? 'Round drawn' : winner + (complete ? ' wins the series!' : ' wins the round');
  return <section className="series-card" aria-label="Series standings">
    <small>{complete ? 'Series champion' : 'Round ' + match.round + ' result'} · First to {wins}</small>
    <div role="status"><h2>{heading}</h2></div>
    <ol>{names.map((name,i) => ({name,slot:i,score:match.scores[i] ?? 0}))
      .sort((a,b)=>b.score-a.score || a.slot-b.slot)
      .map(({name,slot,score})=><li key={slot}><span>{name}</span><b>{score} {score===1 ? 'win':'wins'}</b></li>)}</ol>
    <p>{complete ? 'Returning to the main menu…' : Math.max(...match.scores)>=wins ? 'Final standings up next' : 'Next arena up shortly'}</p>
  </section>;
}
