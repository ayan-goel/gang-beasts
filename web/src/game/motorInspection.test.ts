import { expect, test } from 'vitest';
import { inspectController, type RigControl } from './motorInspection';
import type { RigDefinition } from './rigDefinition';

test('selected body joins native joint order and motor tuning by anatomical name', () => {
  const definition: RigDefinition = {
    schema_version: 1,
    bodies: ['pelvis', 'head', 'left_thigh'].map(part => ({ part, mass: 1, position: [0, 0, 0], shape: { kind: 'sphere', radius: 0.1 } })),
    joints: ['head', 'left_thigh'].map(child => ({ parent: 'pelvis', child, anchor: [0, 0, 0], angular_limits: [[-1, 1], [-1, 1], [-1, 1]] })),
  };
  const control: RigControl = {
    configuration: { model: 'implicit_velocity_pd', joints: ['left_thigh', 'head'].map((child, index) => ({child, stiffness_nm_per_rad: 20, damping_nm_s_per_rad: 1, max_torque_nm: index === 0 ? 10 : 2})) },
    state: () => new Float32Array([1, 1, 0, 0, 0, 2]),
    efforts: () => new Float32Array([0, 1, 0, 3, 4, 0]),
    targets: () => new Float32Array([0, 0, 0, 1, 2, 3]),
    setEnabled: () => {},
  };
  const selected = inspectController(control, definition, 2);
  expect(selected).toMatchObject({ enabled: true, feet: [true, false], balanceEffort: 2, effort: 5, targetVelocity: [1, 2, 3], joint: {child: 'left_thigh', max_torque_nm: 10} });
  expect(inspectController(control, definition, 0)).toMatchObject({joint: undefined, effort: 0, targetVelocity: undefined});
  expect(() => inspectController({...control, targets: () => new Float32Array(3)}, definition, 2)).toThrow('Invalid controller diagnostics');
});
