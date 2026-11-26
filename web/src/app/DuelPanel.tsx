import { useEffect, useRef, useState, type RefObject } from 'react';
import { startDuel, type DuelState } from '../game/startDuel';

const phases = { active: 'Active', knocked_out: 'Knocked out', recovering: 'Recovering', eliminated: 'Out' };
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
    <p className="hint">WASD to move, left Shift to run, Space to jump, Q to headbutt. Tap J/K or a mouse button to punch. Hold to grab. Hold E to lift, then release a hand to let go.</p>
    <h3>Blue controls</h3>
    <p className="hint">Arrow keys to move, right Shift to run, Enter to jump, M to headbutt. Tap comma/period to punch, or hold to grab. Hold slash to lift.</p>
    <dl>
      <div><dt>Coral hits</dt><dd id="coral-hits">{state.hits[0]}</dd></div>
      <div><dt>Blue hits</dt><dd id="blue-hits">{state.hits[1]}</dd></div>
      {colors.map((name, id) => <div key={name}>
        <dt>{name} grips</dt><dd aria-label={`${name} grips`}>{state.grips[id]!.join(' · ') || 'Hands free'}</dd>
      </div>)}
      {colors.map((name, id) => <div key={name}>
        <dt>{name} stamina</dt><dd aria-label={`${name} stamina`}>{Math.round(state.stamina[id]!)}</dd>
      </div>)}
      {state.conditions.map((condition, id) => <div key={id}>
        <dt>{colors[id]} condition</dt>
        <dd aria-label={`${colors[id]} condition`}>{phases[condition.phase]} · {Math.ceil(condition.health)} health</dd>
      </div>)}
      <div><dt>Completed tick</dt><dd id="duel-tick">{state.tick}</dd></div>
    </dl>
    <h3>Last hit</h3>
    <p id="duel-hit">{last ? `${colors[last.attacker]} ${last.action === 'headbutt' ? 'headbutted' : 'hit'} ${colors[last.defender]}'s ${last.region.replaceAll('_', ' ')} for ${last.damage.toFixed(1)} points.` : 'No hits yet.'}</p>
  </section>;
}
