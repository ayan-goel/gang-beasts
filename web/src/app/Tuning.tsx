import { useState } from 'react';
import { downloadJson } from '../game/downloadJson';
import type { InspectorProps } from './types';

export function Tuning({ state, act }: InspectorProps) {
  const configuration = JSON.parse(state.configuration) as { schema_version: number; gravity_y: number; solver_iterations: number };
  const [gravity, setGravity] = useState(String(configuration.gravity_y));
  const [iterations, setIterations] = useState(String(configuration.solver_iterations));
  return <section className="inspector-section" aria-label="Physics tuning">
    <h2>Physics tuning</h2>
    <form onSubmit={event => {
      event.preventDefault();
      act(p => p.configure(JSON.stringify({ ...configuration, gravity_y: Number(gravity), solver_iterations: Number(iterations) })));
    }}>
      <label className="field">Gravity Y (m/s²)
        <input type="number" required min="-30" max="-0.1" step="any" value={gravity} onChange={event => setGravity(event.target.value)} />
      </label>
      <label className="field">Solver iterations
        <input type="number" required min="1" max="32" step="1" value={iterations} onChange={event => setIterations(event.target.value)} />
      </label>
      <button type="submit">Apply tuning</button>
    </form>
    <p className="hint">Applying tuning restarts the fixture paused{state.supportsRecording ? ' and starts a new recording' : ''}.</p>
    <button className="quiet" onClick={() => act(p => downloadJson('physics-v1.json', p.simulation.configuration_json()))}>
      Export configuration
    </button>
  </section>;
}
