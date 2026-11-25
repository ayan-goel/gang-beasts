import { useEffect, useRef, useState } from 'react';
import type { PlaygroundSnapshot } from '../game/Playground';
import { startPlayground, type PlaygroundRuntime } from '../game/startPlayground';
import { Controls } from './Controls';
import { Diagnostics } from './Diagnostics';

export function App() {
  const viewport = useRef<HTMLDivElement>(null);
  const runtime = useRef<PlaygroundRuntime | null>(null);
  const [state, setState] = useState<PlaygroundSnapshot | null>(null);
  const [error, setError] = useState<string | null>(null);
  useEffect(() => {
    try {
      runtime.current = startPlayground(viewport.current!, setState);
    } catch (failure) {
      setError(`Unable to start the playground. WebGL 2 is required. ${String(failure)}`);
    }
    return () => { runtime.current?.dispose(); runtime.current = null; };
  }, []);
  return <>
    <header>
      <div><p className="eyebrow">Physics Brawler</p><h1>Development playground</h1></div>
      <span className="badge">PHYSICS LAB / 03</span>
    </header>
    <div className="workspace">
      <section className="viewport" ref={viewport} aria-label="Physics scene">
        <div className="scene-label"><p className="eyebrow">Test environment</p>
          <h2>Floating platform</h2><p>One body. Real contact. Fixed timestep.</p></div>
      </section>
      <aside aria-label="Playground tools">
        {error ? <p role="alert">{error}</p> : <p role="status">{state ? 'Simulation ready' : 'Loading simulation…'}</p>}
        {state && <>
          <Controls state={state} act={action => {
            try { runtime.current?.act(action); } catch (failure) { setError(String(failure)); }
          }} />
          <Diagnostics state={state} />
        </>}
      </aside>
    </div>
  </>;
}
