import { useEffect, useRef, useState } from 'react';
import { loadSkyline } from '../lobby/content';
import type { MatchView } from '../rendering/SkylineView';
import { startRemoteScene } from './RemoteScene';
import type { MatchFeed } from './MatchFeed';
import './match.css';

const phaseNames = {countdown:'Countdown', playing:'Playing', round_results:'Round results', match_results:'Match results'};
export default function OnlineMatch({feed, exit}: {feed: MatchFeed; exit: () => void}) {
  const container = useRef<HTMLDivElement>(null);
  const [match, setMatch] = useState<MatchView | null>(null);
  const [error, setError] = useState('');
  useEffect(() => {
    let live = true;
    let stop: (() => void) | undefined;
    void loadSkyline().then(content => {
      if (live) stop = startRemoteScene(container.current!, content, feed, setMatch);
    }).catch(() => {if (live) setError('Could not display the arena. Reload to reconnect.');});
    return () => { live = false; stop?.(); };
  }, [feed]);
  const winner = match?.winner == null ? 'Draw' : feed.fighters[match.winner]!.name + ' wins';
  return <section className="online-match" aria-label="Online Skyline arena">
    <div className="online-header"><h1>Skyline Melee</h1><button onClick={exit}>Leave room</button></div>
    <div className="online-scene" ref={container}/>
    {match && <div className="online-score">
      <span aria-label="Round number">Round {match.round}</span>
      <span aria-label="Round phase">{phaseNames[match.phase]}</span>
      {feed.fighters.map((p,i) => <span key={p.slot}>{p.name} <b aria-label={p.name + ' score'}>{match.scores[i]}</b></span>)}
    </div>}
    {match && match.phase !== 'playing' && <div className="online-callout" role="status">{match.phase === 'countdown' ? Math.max(1,Math.ceil(match.remaining/60)) : winner}</div>}
    {error && <p role="alert">{error}</p>}
  </section>;
}
