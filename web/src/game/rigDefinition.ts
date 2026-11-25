import { inspectMovement, type MovementPort } from './movementInspection';
import { inspectController, type RigControl } from './motorInspection';

export type BodyShape =
  | { kind: 'sphere'; radius: number }
  | { kind: 'capsule'; radius: number; half_height: number }
  | { kind: 'rounded_box'; half_extents: [number, number, number]; border_radius: number };

export interface RigDefinition {
  schema_version: 1;
  bodies: { part: string; mass: number; position: [number, number, number]; shape: BodyShape }[];
  joints: { parent: string; child: string; anchor: [number, number, number]; angular_limits: [number, number][] }[];
}

export interface RigInspection {
  readonly controller?: RigControl;
  readonly movement?: MovementPort;
  definition: RigDefinition;
  selectedBody: number;
  selectBody(index: number): void;
  bodyStates(): Float32Array;
  jointStates(): Float32Array;
}

export function inspectRig(rig: RigInspection) {
  const bodies = rig.bodyStates();
  const joints = rig.jointStates();
  let mass = 0;
  let maxAnchorError = 0;
  for (let i = 13; i < bodies.length; i += 14) mass += bodies[i]!;
  for (let i = 16; i < joints.length; i += 17) maxAnchorError = Math.max(maxAnchorError, joints[i]!);
  return {
    controller: rig.controller ? inspectController(rig.controller, rig.definition, rig.selectedBody) : undefined,
    movement: rig.movement ? inspectMovement(rig.movement) : undefined,
    position: [bodies[28]!, bodies[30]!], speed: Math.hypot(bodies[35]!, bodies[37]!),
    selectedBody: rig.selectedBody, bodyCount: bodies.length / 14, jointCount: joints.length / 17,
    mass, maxAnchorError, bodyNames: rig.definition.bodies.map(body => body.part.replaceAll('_', ' ')),
    limits: rig.definition.joints.find(joint => joint.child === rig.definition.bodies[rig.selectedBody]!.part)?.angular_limits,
  };
}
