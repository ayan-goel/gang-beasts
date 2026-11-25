import type { InspectorProps } from './types';

export function RagdollInspector({ state, act }: InspectorProps) {
  const rig = state.rig;
  if (!rig) return null;
  return <section className="inspector-section" aria-label="Ragdoll inspection">
    <h2>Physical character</h2>
    <p className="hint" id="rig-summary">{rig.bodyCount} bodies · {rig.jointCount} joints · {rig.mass.toFixed(1)} kg</p>
    <label className="field">Inspect body
      <select value={rig.selectedBody} onChange={event => act(p => p.simulation.rig?.selectBody(Number(event.target.value)))}>
        {rig.bodyNames.map((name, index) => <option key={name} value={index}>{name}</option>)}
      </select>
    </label>
    <p className="hint">The highlighted body receives impulses and supplies the live body readings.{!rig.controller && ' No motors are active.'}</p>
    <label className="check"><input type="checkbox" checked={state.showJoints}
      onChange={event => act(p => { p.showJoints = event.target.checked; })} />Show joint frames</label>
    {state.showJoints && <p className="hint">Paired anchor dots; X red, Y green, Z blue. Frames: <span id="joint-frame-count">{rig.jointCount}</span>.</p>}
    {state.showJoints && rig.controller && <p className="hint">{rig.movement ? 'Pale axes show the requested gait pose on the completed parent frame.' : 'Pale parent axes show the bind-pose target.'} Saturated child axes show actual orientation.</p>}
    <dl><div><dt>Max anchor separation</dt><dd id="anchor-error">{(rig.maxAnchorError * 1000).toFixed(2)} mm</dd></div></dl>
    <details><summary>Selected joint limits</summary>
      {rig.limits ? <dl>{rig.limits.map(([min, max], axis) => <div key={axis}><dt>{'XYZ'[axis]}</dt>
        <dd>{min === max ? 'Locked' : `${min.toFixed(2)} to ${max.toFixed(2)} rad`}</dd></div>)}</dl>
        : <p className="hint">Pelvis is the root body; it has no parent joint.</p>}
    </details>
  </section>;
}
