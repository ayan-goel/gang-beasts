import type { RigDefinition } from './rigDefinition';

export interface MotorConfiguration {
  model: string;
  joints: { child: string; stiffness_nm_per_rad: number; damping_nm_s_per_rad: number; max_torque_nm: number }[];
}

export interface RigControl {
  configuration: MotorConfiguration;
  state(): Float32Array;
  efforts(): Float32Array;
  targets(): Float32Array;
  poses(): Float32Array;
  setEnabled(enabled: boolean): void;
}

export function inspectController(control: RigControl, definition: RigDefinition, selectedBody: number) {
  const state = control.state();
  const efforts = control.efforts();
  const targets = control.targets();
  if (state.length !== 6 || efforts.length !== definition.joints.length * 3 || targets.length !== efforts.length) throw new Error('Invalid controller diagnostics');
  const part = definition.bodies[selectedBody]!.part;
  const jointIndex = definition.joints.findIndex(joint => joint.child === part);
  const joint = control.configuration.joints.find(motor => motor.child === part);
  return {
    enabled: state[0] === 1,
    feet: [state[1] === 1, state[2] === 1] as const,
    balanceEffort: Math.hypot(state[3]!, state[4]!, state[5]!),
    joint,
    effort: jointIndex < 0 ? 0 : Math.hypot(...efforts.subarray(jointIndex * 3, jointIndex * 3 + 3)),
    targetVelocity: jointIndex < 0 ? undefined : Array.from(targets.subarray(jointIndex * 3, jointIndex * 3 + 3)),
  };
}
