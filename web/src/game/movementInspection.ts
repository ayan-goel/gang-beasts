export interface MovementPort {
  state(): Float32Array;
  stoppingMoment(): Float32Array;
  supportMoment(): Float32Array;
  handStates(): BigUint64Array;
  setHand(side: 0 | 1, held: boolean): void;
  clearHands(): void;
  recoveryState(): Float32Array;
  setJump(held: boolean): void;
  clearJumpInput(): void;
  setInput(x: number, z: number, running: boolean): void;
}

export function inspectMovement(movement: MovementPort) {
  const values = movement.state();
  const moment = movement.stoppingMoment();
  const support = movement.supportMoment();
  if (values.length !== 11 || !values.every(Number.isFinite) || moment.length !== 3 || !moment.every(Number.isFinite) || support.length !== 3 || !support.every(Number.isFinite)) throw new Error('Invalid movement diagnostics');
  const recovery = movement.recoveryState();
  if (recovery.length !== 3 || !recovery.every(Number.isFinite)) throw new Error('Invalid recovery diagnostics');
  const handRecords = movement.handStates();
  const phases = ['Idle', 'Wind-up', 'Strike', 'Recovery', 'Reaching'] as const;
  if (handRecords.length !== 4 || handRecords[0]! > 4n || handRecords[2]! > 4n) throw new Error('Invalid hand diagnostics');
  const hands = [phases[Number(handRecords[0])]!, phases[Number(handRecords[2])]!];
  return { hands, recovering: recovery[0]! > 0, recoveryForce: recovery[1]!, recoveryMoment: recovery[2]!,
    supportMoment: Math.hypot(...support), stoppingMoment: Math.hypot(...moment), acceleration: Math.hypot(...values.subarray(0,3)), supportVelocity: Array.from(values.subarray(3,6)),
    feet: [values[6] === 1, values[7] === 1], phase: values[8]!, turnTorque: values[9]!, strength: values[10]! };
}
