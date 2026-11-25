import type { InspectorProps } from './types';

export function MotorInspector({ state, act }: InspectorProps) {
  const control = state.rig?.controller;
  if (!control) return null;
  const support = control.feet[0] && control.feet[1] ? 'Left + right' : control.feet[0] ? 'Left only' : control.feet[1] ? 'Right only' : 'None';
  const joint = control.joint;
  return <section className="inspector-section" aria-label="Character control">
    <h2>Character controller</h2>
    <label className="check"><input type="checkbox" checked={control.enabled}
      onChange={event => act(p => p.setControlEnabled(event.target.checked))} />Enable character control</label>
    <dl>
      <div><dt>Foot support</dt><dd id="foot-support">{control.enabled ? support : 'Control off'}</dd></div>
      <div><dt>Balance effort</dt><dd>{control.balanceEffort.toFixed(2)} N·m</dd></div>
    </dl>
    <p className="hint">Support used on the last tick. Balance weakens on one foot and stops without support.</p>
    <p className="hint" id="motor-target">{joint ? joint.max_torque_nm === 0 ? 'Fixed wrist · no motor' : state.rig?.movement ? 'Procedural gait · pale axes show target' : 'Bind pose · relative rotation 0 rad' : state.rig?.movement ? 'Root target · upright, facing movement' : 'Root target · upright tilt, free yaw'}</p>
    {joint && <dl>
      <div><dt>Local motor effort</dt><dd id="motor-effort">{control.effort.toFixed(2)} N·m</dd></div>
      <div><dt>Joint effort cap</dt><dd id="motor-cap">{joint.max_torque_nm.toFixed(2)} N·m</dd></div>
      <div><dt>Stiffness</dt><dd>{joint.stiffness_nm_per_rad.toFixed(1)} N·m/rad</dd></div>
      <div><dt>Damping</dt><dd>{joint.damping_nm_s_per_rad.toFixed(2)} N·m·s/rad</dd></div>
    </dl>}
    {joint && joint.max_torque_nm > 0 && <details><summary>Solver velocity target</summary>
      <dl>{control.targetVelocity?.map((value, axis) => <div key={axis}><dt>{'XYZ'[axis]}</dt><dd>{value.toFixed(2)} rad/s</dd></div>)}</dl>
      <p className="hint">Last tick’s correction toward the requested pose, in parent-joint axes. Effort is the final solver substep, not a tick average.</p>
    </details>}
    <button disabled={state.impulsePending} onClick={() => act(p => p.queueImpulse([10, 0, 0]))}>Shove right (10 N·s)</button>
    <p className="hint">One +X impulse on the selected body, queued for the next tick.</p>
  </section>;
}
