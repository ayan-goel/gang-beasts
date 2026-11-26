import type { PreparedArena } from './preparedArena';
import { GraphicsUnavailable } from '../rendering/createSceneRenderer';
import { GripNotices } from '../app/GripNotices';
import { MatchMenu, openMenuFromArena } from '../app/MatchMenu';
import { StaminaMeter } from '../app/StaminaMeter';
import { onlineGripLabels } from './gripLabels';
import { FighterNotices } from '../app/FighterNotices';
import { useMatchSound } from '../audio/SkylineSound';
import { useEffect, useLayoutEffect, useRef, useState } from 'react';
import { canSpectate, followedFighter, type CameraChoice } from './spectatorCamera';
import { SpectatorControls } from './SpectatorControls';
import { hazardWarnings } from '../game/skylineHazards';
import { arenaName, SeriesCard, stageWarning } from '../app/SeriesCard';
import type { SkylineContent } from '../lobby/content';
import type { StreamStatus } from './SnapshotTimeline';
import { matchView, startRemoteScene } from './RemoteScene';
import type { MatchFeed } from './MatchFeed';
import './match.css';

export { PreparedArena } from './preparedArena';

const phaseNames = {countdown:'Countdown', playing:'Playing', round_results:'Round results', match_results:'Match results'};
interface Props {
  feed: MatchFeed;
  content: SkylineContent;
  graphics?: PreparedArena;
  wins?: number;
  highLatency?: boolean;
  exit: () => void;
  leaving?: boolean;
  leaveError?: string;
}

export default function OnlineMatch({feed, content, graphics, exit, wins = 3, highLatency, leaveError, leaving}: Props) {
  const container = useRef<HTMLDivElement>(null);
  const [match, setMatch] = useState(() => matchView(feed.latest!, content));
  const [status, setStatus] = useState<StreamStatus>('buffering');
  const [error, setError] = useState('');
  const [menuPhase, setMenuPhase] = useState<string | null>(null);
  const [graphicsAttempt, setGraphicsAttempt] = useState(0);
  const [choice, setChoice] = useState<CameraChoice | null>(null);
  const cameraChoice = useRef(choice);
  useLayoutEffect(() => { cameraChoice.current = choice; }, [choice]);
  useEffect(() => {
    setError('');
    let stop: (() => void) | undefined;
    let timer: number | undefined;
    // Paint the round introduction before WebGL compiles the arena shaders.
    const frame = requestAnimationFrame(() => {
      timer = window.setTimeout(() => {
        try { stop = startRemoteScene(container.current!, content, feed, setMatch, setStatus, cameraChoice, graphics?.take()); }
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
  }, [feed, content, graphics, graphicsAttempt]);
  useMatchSound(match, status === 'live' || status === 'estimating', match.conditions, match);
  const names = feed.fighters.map(f => f.name);
  const spectator = canSpectate(match, feed.own);
  const name = arenaName(match.arena);
  // A round verdict dismisses the menu so it cannot cover the standings or capture the next round's inputs.
  const phaseKey = match.epoch + '/' + match.phase;
  return <section className="online-match" aria-label={'Online ' + name + ' arena'} data-phase={match.phase}>
    <div className="online-scene" ref={container} onKeyDown={event => openMenuFromArena(event, () => setMenuPhase(phaseKey))}/>
    {match.phase !== 'playing' && <div className="online-callout">
      <SeriesCard match={match} map={match.arena} names={names} wins={wins} />
    </div>}
    {menuPhase === phaseKey && <MatchMenu resume={() => {
      setMenuPhase(null);
      container.current?.querySelector<HTMLCanvasElement>('canvas:not([inert])')?.focus({preventScroll:true});
    }}>
      {spectator && <SpectatorControls names={names} visible={match.visible} target={followedFighter(match, feed.own, choice)}
        choose={slot => setChoice(slot === null ? null : {epoch:match.epoch, slot})}/>}
      <button className="match-menu-leave" onClick={exit} disabled={leaving}>Leave room</button>
      {leaveError && <p role="alert">{leaveError}</p>}
    </MatchMenu>}
    {/* Canvas pixels carry no semantics. Keep a passive text alternative outside the keyboard tab order. */}
    <div className="arena-description">
      <h1>{feed.own === null ? 'Spectating ' + name : 'Playing as ' + names[feed.own]}</h1>
      <span aria-label="Arena">{name}</span>
      <span aria-label="Round number">Round {match.round}</span>
      <span aria-label="Round phase">{phaseNames[match.phase]}</span>
      {(match.phase === 'playing' || match.phase === 'countdown') && names.map((player, i) =>
        <span key={i}>{player} <b aria-label={player + ' score'}>{match.scores[i]}</b></span>)}
      <time aria-label="Round time remaining">{Math.floor(Math.ceil(match.timeLeft/60)/60)}:{String(Math.ceil(match.timeLeft/60)%60).padStart(2,'0')}</time>
      <p role="status" aria-label="Arena hazard">{match.phase === 'playing' && stageWarning(match.arena, match, hazardWarnings[match.hazard])}</p>
      {match.phase === 'playing' && <>
        <FighterNotices names={names} phases={match.conditions} />
        <GripNotices names={names} grips={match.gripTargets.map(targets => onlineGripLabels(targets,names,content.fighter.bodies,match.arena))} />
        {names.map((player,i) => match.visible[i]
          ? <StaminaMeter key={i} name={player} points={match.stamina[i]!} holding={match.gripTargets[i]!.some(Boolean)} />
          : <span key={i} aria-label={player + ' eliminated'}>Out</span>)}
      </>}
      {highLatency && <p>High network delay. Controls may feel delayed.</p>}
    </div>
    {status === 'stalled' && !error && <p className="arena-connection" role="status">Waiting for the server…</p>}
    {error && <div className="arena-connection"><p role="alert">{error}</p>
      <button onClick={() => setGraphicsAttempt(n => n + 1)}>Retry graphics</button>
      <button onClick={exit} disabled={leaving}>Leave room</button>
      {leaveError && <p role="alert">{leaveError}</p>}</div>}
  </section>;
}
