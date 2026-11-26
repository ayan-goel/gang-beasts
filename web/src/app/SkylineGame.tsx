import { useEffect, useRef, useState } from 'react';
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
  const [error, setError] = useState('');
  useEffect(() => {
    try { runtime.current = startDuel(viewport.current!, setState, true); }
    catch (e) { setError(String(e)); }
    return () => { runtime.current?.dispose(); runtime.current = null; };
  }, []);
  const match = state?.match;
  const complete = match?.phase === 'match_results';
  const menu = !started || state?.paused || complete;
  const winner = match?.winner == null ? 'Draw' : `${names[match.winner]} wins the ${complete ? 'match' : 'round'}`;
  const start = () => {
    if (complete) runtime.current?.reset();
    setStarted(true); runtime.current?.toggle();
  };
  const clock = Math.ceil((match?.timeLeft ?? 10800) / 60);
  return <main className="skyline-shell">
    <header className="skyline-header">
      <a className="roof-brand" href="/play/"><span aria-hidden="true" className="roof-mark">▰</span> Skyline Melee</a>
      <span>Local multiplayer</span><a href="/rooms/">Private rooms</a><a href="/">Physics playground ↗</a>
    </header>
    <div className={menu ? 'skyline-layout has-menu' : 'skyline-layout'}>
      {menu && <section className="skyline-menu">
        <div><p className="skyline-kicker">{complete ? 'Match results' : started ? 'Paused' : 'Two players · One keyboard'}</p>
          <h1>{complete ? winner : started ? 'Game paused' : 'Skyline Melee'}</h1>
          <p>Stay on the rooftop. The last fighter standing takes the round. First to three wins.</p></div>
        <div className="skyline-bindings">
          <h2><i className="dot coral" />Coral</h2><p>WASD move · Left Shift run · Space jump / pull up<br />J / K punch or grab · E lift</p>
          <h2><i className="dot blue" />Blue</h2><p>Arrows move · Right Shift run · Enter jump / pull up<br />Comma / period punch or grab · Slash lift</p>
          <p>Hold a hand button to grab a fighter, crate, vent or roof edge. Release during a lift or swing to throw. While hanging, hold jump to pull up. Z / X turns the shared camera.</p>
        </div>
        <button className="skyline-start" disabled={!match} onClick={start}>{complete ? 'Rematch' : started ? 'Resume match' : 'Start match'} <span aria-hidden="true">→</span></button>
      </section>}
      <section className="skyline-stage" aria-label="Skyline arena">
        <div className="scene-canvas" ref={viewport} />
        {match && <div className="match-hud">
          <div className="match-round"><span aria-label="Round number">Round {match.round}</span><strong>{Math.floor(clock / 60)}:{String(clock % 60).padStart(2, '0')}</strong></div>
          <div className="match-score">{names.map((name, id) => <span key={name}><i className={`dot ${name.toLowerCase()}`} />{name}<b aria-label={`${name} score`}>{match.scores[id]}</b></span>)}</div>
          {!menu && <button className="match-pause" onClick={() => runtime.current?.toggle()}>Pause</button>}
        </div>}
        <span className="visually-hidden" aria-label="Round phase">{match && phases[match.phase]}</span>
        {started && !menu && match && <div className="match-callout" aria-live="polite">
          {match.phase === 'countdown' && <strong>{Math.max(1, Math.ceil(match.remaining / 60))}</strong>}
          {match.phase === 'round_results' && <strong>{winner}</strong>}
          {match.phase === 'playing' && hazardWarnings[match.hazard] && <span>{hazardWarnings[match.hazard]}</span>}
          {match.phase === 'playing' && state?.conditions.map((c, i) => c.phase === 'knocked_out' ? <span key={i}>{names[i]} knocked out</span> : null)}
        </div>}
        {started && !menu && match?.phase === 'playing' && <div className="match-grips">
          {state?.grips.map((hands, id) => hands.length > 0 && <div key={id} aria-label={`${names[id]} grips`}>
            <b>{names[id]}</b> {hands.join(' · ')}
          </div>)}
        </div>}
        {error && <p role="alert">{error}</p>}
      </section>
    </div>
  </main>;
}
