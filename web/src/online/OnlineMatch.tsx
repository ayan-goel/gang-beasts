import { useEffect, useRef, useState } from 'react';
import { colors } from '../lobby/api';
import { loadSkyline } from '../lobby/content';
import type { MatchView } from '../rendering/SkylineView';
import type { StreamStatus } from './SnapshotTimeline';
import { startRemoteScene } from './RemoteScene';
import type { MatchFeed } from './MatchFeed';
import './match.css';

const phaseNames = {countdown:'Countdown', playing:'Playing', round_results:'Round results', match_results:'Match results'};
export default function OnlineMatch({feed, exit, rematch}: {feed: MatchFeed; exit: () => void; rematch: (() => void) | undefined}) {
  const container = useRef<HTMLDivElement>(null);
  const [match, setMatch] = useState<MatchView | null>(null);
  const [status, setStatus] = useState<StreamStatus>('buffering');
  const [error, setError] = useState('');
  useEffect(() => {
    let live = true;
    let stop: (() => void) | undefined;
    void loadSkyline().then(content => {
      if (live) stop = startRemoteScene(container.current!, content, feed, setMatch, setStatus);
    }).catch(() => {if (live) setError('Could not display the arena. Reload to reconnect.');});
    return () => { live = false; stop?.(); };
  }, [feed]);
  const winner = match?.winner == null ? 'Draw' : feed.fighters[match.winner]!.name + ' wins';
  return <section className="online-match" aria-label="Online Skyline arena">
    <div className="online-header"><h1>{feed.own === null ? 'Spectating Skyline' : 'Playing as ' + feed.fighters[feed.own]!.name}</h1><button onClick={exit}>Leave room</button></div>
    <div className="online-scene" ref={container}/>
    {match && <div className="online-score">
      <span aria-label="Round number">Round {match.round}</span>
      <span aria-label="Round phase">{phaseNames[match.phase]}</span>
      <time aria-label="Round time remaining">{Math.floor(Math.ceil(match.timeLeft/60)/60)}:{String(Math.ceil(match.timeLeft/60)%60).padStart(2,'0')}</time>
      {feed.fighters.map((p,i) => <span className="online-fighter" key={p.slot}>
        <i className={'swatch color-' + p.color} aria-label={colors[p.color]}/>
        {p.name} <b aria-label={p.name + ' score'}>{match.scores[i]}</b>
        {!match.visible[i] && <small aria-label={p.name + ' eliminated'}>Out</small>}
      </span>)}
    </div>}
    <p className="online-controls">{feed.own === null ? 'Spectating' : 'WASD or arrows to move · Shift to run · Space to jump or pull up · J / K to punch or grab · E to lift'}</p>
    {match && match.phase !== 'playing' && <div className="online-callout"><span role="status">{match.phase === 'countdown' ? Math.max(1,Math.ceil(match.remaining/60)) : winner}</span>
      {match.phase === 'match_results' && rematch && <button className="room-primary" onClick={rematch}>Play again</button>}
    </div>}
    {status === 'stalled' && <p className="online-status" role="status">Waiting for the server…</p>}
    {error && <p role="alert">{error}</p>}
  </section>;
}
