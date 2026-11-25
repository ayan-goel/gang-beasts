import type { PlaybackRate } from '../game/Playground';
import type { InspectorProps } from './types';

export function Controls({ state, act }: InspectorProps) {
  return <section className="controls" aria-label="Playback controls">
    <h2>Playback</h2>
    {!state.movementInput && <><div className="button-row">
      <button className="primary" disabled={state.replayComplete} onClick={() => act(p => p.setPaused(!state.paused))}>
        {state.paused ? 'Resume' : 'Pause'}
      </button>
      <button disabled={!state.paused || state.replayComplete} onClick={() => act(p => p.singleStep())}>Single step</button>
    </div>
    <p className="hint">Pause to advance exactly one physics tick.</p>
    </>}
    <label className="field">Playback speed
      <select value={state.rate} onChange={event => act(p => p.setRate(Number(event.target.value) as PlaybackRate))}>
        <option value={1}>Normal · 1×</option>
        <option value={0.5}>Slow · 0.5×</option>
        <option value={0.25}>Very slow · 0.25×</option>
      </select>
    </label>
    {!state.movementInput && <button onClick={() => act(p => p.reset())}>Reset fixture</button>}
    <h2>Inspect physics</h2>
    <button disabled={state.impulsePending || state.replayEnd !== undefined} onClick={() => act(p => p.queueImpulse())}>
      Apply upward impulse
    </button>
    <p className="hint">4 N·s once on the next tick. {state.impulsePending ? 'Impulse queued.' : ''}</p>
    {state.rig && <><button disabled={state.impulsePending} onClick={() => act(p => p.queueImpulse([0, 0, -4]))}>Apply forward impulse</button>
      <p className="hint">4 N·s toward −Z on the selected body.</p></>}
    <label className="check"><input type="checkbox" checked={state.showColliders}
      onChange={event => act(p => { p.showColliders = event.target.checked; })} />Show colliders</label>
  </section>;
}
