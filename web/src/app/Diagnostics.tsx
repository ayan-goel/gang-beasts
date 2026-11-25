import type { PlaygroundSnapshot } from '../game/Playground';

export function Diagnostics({ state }: { state: PlaygroundSnapshot }) {
  return <section aria-label="Simulation diagnostics">
    <h2>Live simulation</h2>
    <dl>
      <div><dt>Completed tick</dt><dd id="tick">{state.tick}</dd></div>
      <div><dt>Body height</dt><dd id="height">{state.height.toFixed(3)} m</dd></div>
      <div><dt>Vertical speed</dt><dd id="vertical-speed">{state.verticalSpeed.toFixed(3)} m/s</dd></div>
      <div><dt>Body mass</dt><dd>{state.mass.toFixed(1)} kg</dd></div>
      <div><dt>Physics step</dt><dd>{state.stepMs.toFixed(2)} ms</dd></div>
      <div><dt>Discarded time</dt><dd>{state.droppedSeconds.toFixed(2)} s</dd></div>
      {state.showColliders && <div><dt>Collider segments</dt><dd id="debug-segments">{state.debugSegments}</dd></div>}
    </dl>
    <p className="note">Rust / Rapier · WebAssembly<br />Fixed physics step: 1/60 second.<br />
      A diagnostic fixture; movement and combat are still in development.</p>
  </section>;
}
