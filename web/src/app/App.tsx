import { LedgePracticePanel } from './LedgePracticePanel';
import { DuelPanel } from './DuelPanel';
import { MovementControls } from './MovementControls';
import { MovementInspector } from './MovementInspector';
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

type SceneChoice = Scenario | 'duel' | 'ledge';

const environments: Record<SceneChoice, { name: string; title: string; description: string }> = {
  cube: { name: 'Cuboid fixture', title: 'Floating platform', description: 'One body. Real contact. Fixed timestep.' },
  ragdoll: { name: 'Passive ragdoll', title: 'Passive ragdoll', description: 'Fifteen bodies. Physical joints. No motors.' },
  standing: { name: 'Standing ragdoll', title: 'Standing ragdoll', description: 'Bounded motors. Foot support. Physical balance.' },
  locomotion: { name: 'Locomotion', title: 'Locomotion playground', description: 'Walk. Run. Turn. Keep your balance.' },
  duel: { name: 'Local sparring', title: 'Local sparring', description: 'Two fighters · Shared keyboard' },
  practice: { name: 'Strike practice', title: 'Strike practice', description: 'Punch the suspended pad with either hand.' },
  ledge: { name: 'Ledge rescue', title: 'West walkway', description: 'Catch the roof as the walkway falls.' },
  stairs: { name: 'Low stairs', title: 'Low stairs', description: 'Walk forward to approach three 10 cm risers.' },
};

export function App() {
  const viewport = useRef<HTMLDivElement>(null);
  const runtime = useRef<PlaygroundRuntime | null>(null);
  const generation = useRef(0);
  const [scenario, setScenario] = useState<SceneChoice>(() => new URLSearchParams(location.search).get('scene') === 'ledge' ? 'ledge' : 'cube');
  const [state, setState] = useState<PlaygroundSnapshot | null>(null);
  const [error, setError] = useState<string | null>(null);
  const fixture = scenario !== 'duel' && scenario !== 'ledge';
  useEffect(() => {
    generation.current++;
    setState(null);
    setError(null);
    if (!fixture) return;
    try {
      runtime.current = startPlayground(viewport.current!, setState, scenario);
    } catch (failure) {
      setError(`Unable to start the playground. ${String(failure)}`);
    }
    return () => { runtime.current?.dispose(); runtime.current = null; };
  }, [scenario, fixture]);
  const act: InspectorProps['act'] = action => {
    try { runtime.current?.act(action); setError(null); }
    catch (failure) { setError(String(failure)); }
  };
  return <>
    <header>
      <div><p className="eyebrow">Physics Brawler</p><h1>Development playground</h1></div>
      <a className="play-link" href="/play/">Play Skyline Melee →</a>
    </header>
    <div className="workspace">
      <section className="viewport" aria-label="Physics scene">
        <div className="scene-canvas" ref={viewport} />
        <div className="scene-label"><p className="eyebrow">Test environment</p>
          <h2>{environments[scenario].title}</h2>
          <p>{environments[scenario].description}</p></div>
        {fixture && state && <MovementControls state={state} act={act} focus={()=>viewport.current?.querySelector('canvas')?.focus({preventScroll:true})} />}
      </section>
      <aside aria-label="Playground tools">
        {fixture && (error ? <p role="alert">{error}</p> : <p role="status">{state ? 'Simulation ready' : 'Loading simulation…'}</p>)}
        <label className="field">Test scene
          <select value={scenario} onChange={event => setScenario(event.target.value as SceneChoice)}>
            {Object.entries(environments).map(([key, scene]) => <option key={key} value={key}>{scene.name}</option>)}
          </select>
        </label>
        {scenario === 'ledge' && <LedgePracticePanel viewport={viewport} />}
        {scenario === 'duel' && <DuelPanel viewport={viewport} />}
        {fixture && state && <>
          <MovementInspector state={state} act={act} />
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
