import { GraphicsUnavailable } from '../rendering/createSceneRenderer';
import { pullHint } from '../game/ledgeHint';
import { DisplaySettings } from './DisplaySettings';
import { MatchMenu, openMenuFromArena } from './MatchMenu';
import { Scoreboard } from './Scoreboard';
import { meleeResult } from '../game/meleeResult';
import { GripNotices } from './GripNotices';
import { StaminaMeter } from './StaminaMeter';
import { FighterNotices } from './FighterNotices';
import { MuteButton, useMatchSound } from '../audio/SkylineSound';
import { useEffect, useRef, useState } from 'react';
import { loadFighterModel } from '../rendering/fighter/loadFighterModel';
import { startDuel, type DuelState } from '../game/startDuel';
import { hazardWarnings } from '../game/skylineHazards';
import './skyline.css';

const names = ['Coral', 'Blue'];
const phases = { countdown: 'Countdown', playing: 'Playing', round_results: 'Round results', match_results: 'Match results' };

export function SkylineGame() {
  const viewport = useRef<HTMLDivElement>(null);
  const runtime = useRef<ReturnType<typeof startDuel> | null>(null);
  const [state, setState] = useState<DuelState>();
  const [started, setStarted] = useState(false);
  const [error, setError] = useState<{message: string; retry: string}>();
  const [attempt, setAttempt] = useState(0);
  useEffect(() => {
    let active = true;
    setError(undefined);
    void loadFighterModel().then(model => {
      if (!active) return;
      try { runtime.current = startDuel(viewport.current!, setState, model); }
      catch (e) { setError({message:e instanceof GraphicsUnavailable ? e.message : String(e), retry:'Retry graphics'}); }
    }).catch(() => { if (active) setError({message:'Could not load the fighters. Check your connection and try again.', retry:'Retry loading fighters'}); });
    return () => { active = false; runtime.current?.dispose(); runtime.current = null; };
  }, [attempt]);
  const match = state?.match;
  const complete = match?.phase === 'match_results';
  const menu = !started || state?.paused || complete;
  const conditions = state?.conditions.map(c => c.phase) ?? [];
  useMatchSound(match, started && !state?.paused, conditions, state);
  const winner = match && meleeResult(match, names);
  const start = () => {
    if (complete) runtime.current?.reset();
    setStarted(true); runtime.current?.toggle();
  };
  const clock = Math.ceil((match?.timeLeft ?? 10800) / 60);
  const showIntro = !started || complete;
  const controls = <div className="skyline-bindings">
    <h2><i className="dot coral" />Coral</h2><p>WASD move · Left Shift run · Space jump / pull up<br />J / K punch or grab · E lift · Q headbutt</p>
    <h2><i className="dot blue" />Blue</h2><p>Arrows move · Right Shift run · Enter jump / pull up<br />Comma / period punch or grab · Slash lift · M headbutt</p>
    <p>Hold a hand button to grab a fighter, crate, vent or roof edge. Release during a lift or swing to throw. While hanging, hold jump to pull up. Z / X turns the shared camera. Tab shows the scoreboard. Escape pauses the match from the arena.</p>
  </div>;
  return <main className="skyline-shell">
    <header className="skyline-header">
      <a className="roof-brand" href="/" aria-label="Main menu"><span aria-hidden="true" className="roof-mark">▰</span> Skyline Melee</a>
      <span>Local multiplayer</span><DisplaySettings /><MuteButton /><a href="/rooms/">Private rooms</a><a href="/playground/">Physics playground ↗</a>
    </header>
    {started && !complete && state?.paused && <MatchMenu local resume={start}>{controls}</MatchMenu>}
    <div className={showIntro ? 'skyline-layout has-menu' : 'skyline-layout'}>
      {showIntro && <section className="skyline-menu">
        <div><p className="skyline-kicker">{complete ? 'Match results' : 'Two players · One keyboard'}</p>
          <h1>{complete ? winner : 'Skyline Melee'}</h1>
          <p>Stay on the rooftop. The last fighter standing takes the round. First to three wins.</p></div>
        {controls}
        <button className="skyline-start" disabled={!match} onClick={start}>{complete ? 'Rematch' : 'Start match'} <span aria-hidden="true">→</span></button>
        {error && <div><p role="alert">{error.message}</p><button onClick={() => setAttempt(n => n + 1)}>{error.retry}</button></div>}
      </section>}
      <section className="skyline-stage" aria-label="Skyline arena">
        <div className="scene-canvas" ref={viewport}
          onKeyDown={event => { if (!menu) openMenuFromArena(event, () => runtime.current?.toggle()); }} />
        {match && <div className="match-hud">
          <Scoreboard className="local-scoreboard" viewport={viewport}>
            <div className="match-round"><span aria-label="Round number">Round {match.round}</span><strong aria-label="Round time remaining">{Math.floor(clock / 60)}:{String(clock % 60).padStart(2, '0')}</strong></div>
            <div className="match-score">{names.map((name, id) => <div key={name}>
              <span className="fighter-score"><i className={`dot ${name.toLowerCase()}`} />{name}<b aria-label={`${name} score`}>{match.scores[id]}</b></span>
              {!menu && match.phase === 'playing' && state && match.visible[id] &&
                <StaminaMeter name={name} points={state.stamina[id]!} holding={state.grips[id]!.length > 0} />}
            </div>)}</div>
          </Scoreboard>
          {!menu && <button className="match-pause" aria-keyshortcuts="Escape" onClick={() => runtime.current?.toggle()}>Pause</button>}
        </div>}
        <span className="visually-hidden" aria-label="Round phase">{match && phases[match.phase]}</span>
        {started && !menu && match && <div className="match-callout" aria-live="polite">
          {match.phase === 'countdown' && <strong>{Math.max(1, Math.ceil(match.remaining / 60))}</strong>}
          {match.phase === 'round_results' && <strong>{winner}</strong>}
          {match.phase === 'playing' && hazardWarnings[match.hazard] && <span>{hazardWarnings[match.hazard]}</span>}
          {match.phase === 'playing' && <FighterNotices names={names} phases={conditions} />}
        </div>}
        {started && !menu && match?.phase === 'playing' && <div className="match-grips">
          {state && <GripNotices names={names} grips={state.grips}
            hints={state.fixedLedge.map((onLedge, slot) => onLedge && conditions[slot] === 'active' ? pullHint(slot === 0 ? 'Space' : 'Enter') : '')} />}
        </div>}
      </section>
    </div>
  </main>;
}
