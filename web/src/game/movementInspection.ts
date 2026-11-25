export interface MovementPort {
  state(): Float32Array;
  setInput(x: number, z: number, running: boolean): void;
}

export function inspectMovement(movement: MovementPort) {
  const values = movement.state();
  if (values.length !== 11 || !values.every(Number.isFinite)) throw new Error('Invalid movement diagnostics');
  return { acceleration: Math.hypot(...values.subarray(0,3)), supportVelocity: Array.from(values.subarray(3,6)),
    feet: [values[6] === 1, values[7] === 1], phase: values[8]!, turnTorque: values[9]!, strength: values[10]! };
}
