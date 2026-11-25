export const inputKind = 1;
export const protocolVersion = 1;
export const inputBytes = 18;
export const buttons = {
  jump: 1, run: 2, leftHand: 4, rightHand: 8, lift: 16, headbutt: 32,
  kick: 64, crouch: 128, emote: 256, interact: 512,
} as const;

export interface InputFrame {
  sequence: number;
  tick: number;
  axes: readonly [number, number];
  held: number;
  pressed: number;
  released: number;
}

function integer(value: number, low: number, high: number) {
  if (!Number.isInteger(value) || value < low || value > high) throw new RangeError('input field out of range');
}

export function encodeInput(frame: InputFrame): Uint8Array {
  integer(frame.sequence, 0, 0xffffffff);
  integer(frame.tick, 0, 0xffffffff);
  if (frame.axes.length !== 2) throw new RangeError('two movement axes required');
  for (const axis of frame.axes) integer(axis, -127, 127);
  for (const mask of [frame.held, frame.pressed, frame.released]) integer(mask, 0, 0x03ff);
  const { held, pressed, released } = frame;
  if ((pressed & ~released & ~held) || (released & ~pressed & held)) throw new RangeError('inconsistent input edges');

  const bytes = new Uint8Array(inputBytes);
  const view = new DataView(bytes.buffer);
  view.setUint8(0, inputKind);
  view.setUint8(1, protocolVersion);
  view.setUint32(2, frame.sequence, true);
  view.setUint32(6, frame.tick, true);
  view.setInt8(10, frame.axes[0]);
  view.setInt8(11, frame.axes[1]);
  view.setUint16(12, held, true);
  view.setUint16(14, pressed, true);
  view.setUint16(16, released, true);
  return bytes;
}

export function quantizeDirection(x: number, z: number): [number, number] {
  if (!Number.isFinite(x) || !Number.isFinite(z)) throw new RangeError('nonfinite movement');
  const scale = Math.max(1, Math.abs(x), Math.abs(z));
  const a = x / scale, b = z / scale;
  const length = Math.max(1, Math.hypot(a, b));
  const component = (v: number) => Math.sign(v) * Math.round(127 * Math.abs(v) / length) || 0;
  return [component(a), component(b)];
}
