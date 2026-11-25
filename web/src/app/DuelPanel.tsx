import { useEffect, useRef, useState, type RefObject } from 'react';
import { startDuel, type DuelState } from '../game/startDuel';

const colors = ['Coral', 'Blue'] as const;

export function DuelPanel({ viewport }: { viewport: RefObject<HTMLDivElement | null> }) {
  const runtime = useRef<ReturnType<typeof startDuel> | null>(null);
  const [state, setState] = useState<DuelState>();
  const [error, setError] = useState<string>();
  useEffect(() => {
    try { runtime.current = startDuel(viewport.current!, setState); }
    catch (failure) { setError(String(failure)); }
    return () => { runtime.current?.dispose(); runtime.current = null; };
  }, [viewport]);
  if (error) return <p role="alert">{error}</p>;
  if (!state) return <p role="status">Loading fighters…</p>;
  const last = state.latest;
  return <section className="inspector-section" aria-label="Sparring controls">
    <p role="status">Simulation ready</p>
    <div className="button-row">
      <button className="primary" onClick={() => runtime.current?.toggle()}>{state.paused ? 'Resume' : 'Pause'}</button>
      <button onClick={() => runtime.current?.reset()}>Reset encounter</button>
    </div>
    <h3>Coral controls</h3>
    <p className="hint">WASD to move, left Shift to run, Space to jump. Tap J/K or a mouse button to punch. Hold to grab and release to let go.</p>
    <h3>Blue controls</h3>
    <p className="hint">Arrow keys to move, right Shift to run, Enter to jump. Tap comma/period to punch, or hold to grab.</p>
    <dl>
      <div><dt>Coral hits</dt><dd id="coral-hits">{state.hits[0]}</dd></div>
      <div><dt>Blue hits</dt><dd id="blue-hits">{state.hits[1]}</dd></div>
      {colors.map((name, id) => <div key={name}>
        <dt>{name} grips</dt><dd aria-label={`${name} grips`}>{state.grips[id]!.join(' · ') || 'Hands free'}</dd>
      </div>)}
      <div><dt>Completed tick</dt><dd id="duel-tick">{state.tick}</dd></div>
    </dl>
    <h3>Last hit</h3>
    <p id="duel-hit">{last ? `${colors[last.attacker]} hit ${colors[last.defender]}'s ${last.region.replaceAll('_', ' ')} for ${last.damage.toFixed(1)} points.` : 'No hits yet.'}</p>
  </section>;
}
