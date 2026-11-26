import { GraphicsUnavailable } from '../rendering/createSceneRenderer';
import { GripNotices } from '../app/GripNotices';
import { StaminaMeter } from '../app/StaminaMeter';
import { onlineGripLabels } from './gripLabels';
import { gripHint } from './gripHint';
import { FighterNotices } from '../app/FighterNotices';
import { useMatchSound } from '../audio/SkylineSound';
import { useEffect, useLayoutEffect, useRef, useState } from 'react';
import { canSpectate, followedFighter, type CameraChoice } from './spectatorCamera';
import { SpectatorControls } from './SpectatorControls';
import { colors } from '../lobby/api';
import { hazardWarnings } from '../game/skylineHazards';
import { meleeResult } from '../game/meleeResult';
import type { SkylineContent } from '../lobby/content';
import type { StreamStatus } from './SnapshotTimeline';
import { matchView, startRemoteScene } from './RemoteScene';
import type { MatchFeed } from './MatchFeed';
import './match.css';

const phaseNames = {countdown:'Countdown', playing:'Playing', round_results:'Round results', match_results:'Match results'};
export default function OnlineMatch({feed, content, exit, rematch, highLatency}: {feed: MatchFeed; highLatency?: boolean; content: SkylineContent; exit: () => void; rematch: (() => void) | undefined}) {
  const container = useRef<HTMLDivElement>(null);
  const [match, setMatch] = useState(() => matchView(feed.latest!, content));
  const [status, setStatus] = useState<StreamStatus>('buffering');
  const [error, setError] = useState('');
  const [graphicsAttempt, setGraphicsAttempt] = useState(0);
  const [choice, setChoice] = useState<CameraChoice | null>(null);
  const cameraChoice = useRef(choice);
  useLayoutEffect(() => { cameraChoice.current = choice; }, [choice]);
  useEffect(() => {
    setError('');
    let stop: (() => void) | undefined;
    let timer: number | undefined;
    // Give the authoritative scoreboard a paint before WebGL compiles the arena shaders.
    const frame = requestAnimationFrame(() => {
      timer = window.setTimeout(() => {
        try { stop = startRemoteScene(container.current!, content, feed, setMatch, setStatus, cameraChoice); }
        catch (failure) {
          setError(failure instanceof GraphicsUnavailable ? failure.message : 'Could not display the arena.');
          // The room keeps playing even when this device cannot draw its first frame.
          const scoreboard = () => { if (feed.latest) setMatch(matchView(feed.latest, content)); };
          feed.onSnapshot = scoreboard;
          scoreboard();
          stop = () => { if (feed.onSnapshot === scoreboard) feed.onSnapshot = undefined; };
        }
      }, 0);
    });
    return () => { cancelAnimationFrame(frame); clearTimeout(timer); stop?.(); };
  }, [feed, content, graphicsAttempt]);
  useMatchSound(match, status === 'live' || status === 'estimating', match.conditions, match);
  const names = feed.fighters.map(f => f.name);
  const spectator = canSpectate(match, feed.own);
  const winner = meleeResult(match, names);
  const hint = feed.own !== null && match.phase === 'playing' && match.conditions[feed.own] === 'active'
    ? gripHint(feed.own, match.gripTargets, match.hazard) : '';
  return <section className="online-match" aria-label="Online Skyline arena">
    <div className="online-header"><h1>{feed.own === null ? 'Spectating Skyline' : 'Playing as ' + feed.fighters[feed.own]!.name}</h1><button onClick={exit}>Leave room</button></div>
    <div className="online-stage">
      <div className="online-scene" ref={container}/>
      <div className="online-notices">
        {highLatency && status !== 'stalled' && <p className="online-delay" role="status">High network delay. Controls may feel delayed.</p>}
        <p className="online-hazard" role="status" aria-label="Arena hazard">
          {match.phase === 'playing' ? hazardWarnings[match.hazard] : ''}
        </p>
        <div className="online-conditions" aria-live="polite">
          {match.phase === 'playing' && <FighterNotices names={names} phases={match.conditions} />}
        </div>
      </div>
    </div>
    <div className="online-score">
      <span aria-label="Round number">Round {match.round}</span>
      <span aria-label="Round phase">{phaseNames[match.phase]}</span>
      <time aria-label="Round time remaining">{Math.floor(Math.ceil(match.timeLeft/60)/60)}:{String(Math.ceil(match.timeLeft/60)%60).padStart(2,'0')}</time>
      {feed.fighters.map((p,i) => <div key={p.slot}>
        <span className="online-fighter">
          <i className={'swatch color-' + p.color} aria-label={colors[p.color]}/>
          {p.name} <b aria-label={p.name + ' score'}>{match.scores[i]}</b>
          {!match.visible[i] && <small aria-label={p.name + ' eliminated'}>Out</small>}
        </span>
        {match.phase === 'playing' && match.visible[i] &&
          <StaminaMeter name={p.name} points={match.stamina[i]!} holding={match.gripTargets[i]!.some(Boolean)} />}
      </div>)}
    </div>
    {match.phase === 'playing' && <div className="online-grips" aria-live="polite">
      <GripNotices names={names} grips={match.gripTargets.map(targets => onlineGripLabels(targets,names,content.fighter.bodies,content.map))} />
    </div>}
    {spectator && <SpectatorControls names={names} visible={match.visible} target={followedFighter(match, feed.own, choice)}
      choose={slot => setChoice(slot === null ? null : {epoch:match.epoch, slot})}/>}
    <p className="online-controls">{hint || (spectator || feed.own === null ? 'Spectating until the next round · Z / X to turn the camera' : 'WASD or arrows to move · Shift to run · Space to jump or pull up · J / K to punch or grab · E to lift · Q to headbutt · Z / X to turn the camera')}</p>
    {match.phase !== 'playing' && <div className="online-callout"><span role="status">{match.phase === 'countdown' ? Math.max(1,Math.ceil(match.remaining/60)) : winner}</span>
      {match.phase === 'match_results' && rematch && <button className="room-primary" onClick={rematch}>Play again</button>}
    </div>}
    {status === 'stalled' && <p className="online-status" role="status">Waiting for the server…</p>}
    {error && <div><p role="alert">{error}</p><button onClick={() => setGraphicsAttempt(n => n + 1)}>Retry graphics</button></div>}
  </section>;
}
