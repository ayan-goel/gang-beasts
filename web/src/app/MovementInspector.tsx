import type { InspectorProps } from './types';

export function MovementInspector({state}: InspectorProps) {
  const movement = state.rig?.movement;
  if (!movement || !state.movementInput) return null;
  const [x,z,run] = state.movementInput.command;
  return <section className="inspector-section" aria-label="Movement diagnostics">
    <h2>Locomotion</h2>
    <dl>
      <div><dt>Input</dt><dd id="movement-command">{x || z ? run ? 'Run' : 'Walk' : 'Idle'}</dd></div>
      <div><dt>Position X / Z</dt><dd><span id="position-x">{state.rig!.position[0]!.toFixed(2)}</span> / <span id="position-z">{state.rig!.position[1]!.toFixed(2)}</span> m</dd></div>
      <div><dt>Pelvis speed</dt><dd>{state.rig!.speed.toFixed(2)} m/s</dd></div>
      <div><dt>Commanded acceleration</dt><dd>{movement.acceleration.toFixed(2)} m/s²</dd></div>
      <div><dt>Support moment</dt><dd id="support-moment">{movement.supportMoment.toFixed(2)} N·m</dd></div>
      <div><dt>Stopping moment</dt><dd id="stopping-moment">{movement.stoppingMoment.toFixed(2)} N·m</dd></div>
      <div><dt>Left / right hand</dt><dd><span id="left-hand-phase">{movement.hands[0]}</span> / <span id="right-hand-phase">{movement.hands[1]}</span></dd></div>
      <div><dt>Get-up assistance</dt><dd id="recovery-status">{movement.recovering ? 'Getting up' : 'Inactive'}</dd></div>
      <div><dt>Active motor strength</dt><dd id="movement-strength">{(movement.strength*100).toFixed(0)}%</dd></div>
    </dl>
    <p className="hint">Direction uses world axes: forward −Z, right +X. This finite platform has open edges. Reset after a fall.</p>
    <details><summary>Gait and support</summary><dl>
      <div><dt>Phase</dt><dd>{movement.phase.toFixed(2)} rad</dd></div>
      <div><dt>Get-up force / moment</dt><dd>{movement.recoveryForce.toFixed(1)} N / {movement.recoveryMoment.toFixed(1)} N·m</dd></div>
      <div><dt>Yaw effort</dt><dd>{movement.turnTorque.toFixed(2)} N·m</dd></div>
      <div><dt>Support velocity</dt><dd>{movement.supportVelocity.map(value=>value.toFixed(2)).join(' / ')} m/s</dd></div>
    </dl></details>
  </section>;
}
