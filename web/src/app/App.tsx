import { useEffect, useRef, useState } from 'react';
import type { PlaygroundSnapshot } from '../game/Playground';
import { startPlayground, type PlaygroundRuntime, type Scenario } from '../game/startPlayground';
import { Controls } from './Controls';
import { Diagnostics } from './Diagnostics';
import { Recordings } from './Recordings';
import { Tuning } from './Tuning';
import type { InspectorProps } from './types';
import { RagdollInspector } from './RagdollInspector';
import { MotorInspector } from './MotorInspector';

export function App() {
  const viewport = useRef<HTMLDivElement>(null);
  const runtime = useRef<PlaygroundRuntime | null>(null);
  const generation = useRef(0);
  const [scenario, setScenario] = useState<Scenario>('cube');
  const [state, setState] = useState<PlaygroundSnapshot | null>(null);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    generation.current++;
    setState(null);
    setError(null);
    try {
      runtime.current = startPlayground(viewport.current!, setState, scenario);
    } catch (failure) {
      setError(`Unable to start the playground. ${String(failure)}`);
    }
    return () => { runtime.current?.dispose(); runtime.current = null; };
  }, [scenario]);
  const act: InspectorProps['act'] = action => {
    try { runtime.current?.act(action); setError(null); }
    catch (failure) { setError(String(failure)); }
  };
  return <>
    <header>
      <div><p className="eyebrow">Physics Brawler</p><h1>Development playground</h1></div>
      <span className="badge">PHYSICS LAB / 05</span>
    </header>
    <div className="workspace">
      <section className="viewport" ref={viewport} aria-label="Physics scene">
        <div className="scene-label"><p className="eyebrow">Test environment</p>
          <h2>{scenario === 'standing' ? 'Standing ragdoll' : scenario === 'ragdoll' ? 'Passive ragdoll' : 'Floating platform'}</h2>
          <p>{scenario === 'standing' ? 'Bounded motors. Foot support. Physical balance.' : scenario === 'ragdoll' ? 'Fifteen bodies. Physical joints. No motors.' : 'One body. Real contact. Fixed timestep.'}</p></div>
      </section>
      <aside aria-label="Playground tools">
        {error ? <p role="alert">{error}</p> : <p role="status">{state ? 'Simulation ready' : 'Loading simulation…'}</p>}
        <label className="field">Test scene
          <select value={scenario} onChange={event => setScenario(event.target.value as Scenario)}>
            <option value="cube">Cuboid fixture</option><option value="ragdoll">Passive ragdoll</option>
            <option value="standing">Standing ragdoll</option>
          </select>
        </label>
        {state && <>
          <RagdollInspector state={state} act={act} />
          <Controls state={state} act={act} />
          <MotorInspector state={state} act={act} />
          <Diagnostics state={state} />
          <Tuning key={`${scenario}:${state.configuration}`} state={state} act={act} />
          {state.supportsRecording && <Recordings state={state} act={act} reportError={setError}
            sessionRevision={() => `${generation.current}:${runtime.current?.playground.sessionRevision ?? -1}`} />}
        </>}
      </aside>
    </div>
  </>;
}
