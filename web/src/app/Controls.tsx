import type { Playground, PlaygroundSnapshot, PlaybackRate } from '../game/Playground';
import { exportConfiguration } from '../game/exportConfiguration';

interface Props {
  state: PlaygroundSnapshot;
  act: (action: (playground: Playground) => void) => void;
}

export function Controls({ state, act }: Props) {
  return <section className="controls" aria-label="Playback controls">
    <h2>Playback</h2>
    <div className="button-row">
      <button className="primary" onClick={() => act(p => p.setPaused(!state.paused))}>
        {state.paused ? 'Resume' : 'Pause'}
      </button>
      <button disabled={!state.paused} onClick={() => act(p => p.singleStep())}>Single step</button>
    </div>
    <p className="hint">Pause to advance exactly one physics tick.</p>
    <label className="field">Playback speed
      <select value={state.rate} onChange={event => act(p => p.setRate(Number(event.target.value) as PlaybackRate))}>
        <option value={1}>Normal · 1×</option>
        <option value={0.5}>Slow · 0.5×</option>
        <option value={0.25}>Very slow · 0.25×</option>
      </select>
    </label>
    <button onClick={() => act(p => p.reset())}>Reset fixture</button>
    <h2>Inspect physics</h2>
    <button disabled={state.impulsePending} onClick={() => act(p => p.queueImpulse())}>
      Apply upward impulse
    </button>
    <p className="hint">4 N·s once on the next tick. {state.impulsePending ? 'Impulse queued.' : ''}</p>
    <label className="check"><input type="checkbox" checked={state.showColliders}
      onChange={event => act(p => { p.showColliders = event.target.checked; })} />Show colliders</label>
    <button className="quiet" onClick={exportConfiguration}>Export configuration</button>
  </section>;
}
