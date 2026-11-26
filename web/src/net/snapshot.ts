import { compactRotation } from './compactRotation';

type Vec3 = [number, number, number];
type Quat = [number, number, number, number];

export interface BodyPose { position: Vec3; rotation: Quat }
export interface FighterState {
  consciousness: number;
  score: number;
  hands: [number, number];
  health: number;
  stamina: number;
  grips: [number, number];
}
export interface Snapshot {
  tick: number;
  applied: number | null;
  round: bigint;
  number: number;
  arenaTick: number;
  phase: number;
  winner: number | null;
  remaining: number;
  bodies: Map<number, BodyPose>;
  fighters: FighterState[];
}

function requireField(valid: boolean, field: string): void {
  if (!valid) throw new RangeError('Invalid full snapshot ' + field);
}

export function decodeSnapshot(bytes: Uint8Array): Snapshot {
  requireField(bytes.length >= 20, 'header');
  const data = new DataView(bytes.buffer, bytes.byteOffset, bytes.byteLength);
  const u8 = (offset: number) => data.getUint8(offset);
  const u16 = (offset: number) => data.getUint16(offset, true);
  const u32 = (offset: number) => data.getUint32(offset, true);
  const f32 = (offset: number) => data.getFloat32(offset, true);
  const flags = u16(2), count = u16(16);
  const packed = (flags & 4) !== 0, acknowledged = (flags & 2) !== 0;
  const stride = packed ? 23 : 32, gameplay = 20 + stride * count;
  requireField(u8(0) === 2 && u8(1) === 1 && (flags & ~6) === 1, 'kind or flags');
  requireField(u32(12) === 0 && (acknowledged || u32(8) === 0), 'baseline or acknowledgement');
  // Check all bounds before allocating per-body state or reading the gameplay block.
  requireField(count <= 256 && u16(18) >= 24 && gameplay + u16(18) === bytes.length, 'length');
  const slots = u8(gameplay + 3), phase = u8(gameplay + 1), winner = u8(gameplay + 2);
  const round = data.getBigUint64(gameplay + 8, true), number = u32(gameplay + 16);
  requireField(u8(gameplay) === 1 && slots >= 2 && slots <= 8 && u16(18) === 24 + slots * 24 && count >= slots * 15, 'roster');
  requireField(phase <= 3 && (winner === 255 || (winner < slots && phase >= 2))
    && u16(gameplay + 6) === 0 && round > 0n && number > 0, 'match');

  const bodies = new Map<number, BodyPose>();
  let previous = 0;
  for (let i = 0; i < count; i++) {
    const at = 20 + stride * i, id = u32(at);
    const expected = 1 + 16 * Math.floor(i / 15) + i % 15;
    requireField(id > previous && (i < slots * 15 ? id === expected : id > 65536 && id <= 131071), 'body ID');
    const position: Vec3 = [f32(at + 4), f32(at + 8), f32(at + 12)];
    const rotation: Quat = packed ? compactRotation(data, at + 16) : [f32(at + 16), f32(at + 20), f32(at + 24), f32(at + 28)];
    const norm = rotation.reduce((sum, v) => sum + v * v, 0);
    requireField(position.every(Number.isFinite) && norm >= 0.998 && norm <= 1.002, 'transform');
    bodies.set(id, { position, rotation });
    previous = id;
  }
  const fighters: FighterState[] = [];
  for (let slot = 0; slot < slots; slot++) {
    const at = gameplay + 24 * (1 + slot);
    const hands: [number, number] = [u8(at + 3), u8(at + 4)];
    const grips: [number, number] = [u32(at + 16), u32(at + 20)];
    const consciousness = u8(at + 1), health = f32(at + 8), stamina = f32(at + 12);
    requireField(u8(at) === slot && consciousness <= 3 && u8(at + 5) === 0 && u16(at + 6) === 0
      && [health, stamina].every(v => v >= 0 && v <= 100) && hands.every(p => p <= 5), 'fighter');
    for (const [side, target] of grips.entries()) {
      const ownLimb = target >= 1 + 16 * slot && target <= 15 + 16 * slot;
      requireField((hands[side] === 5) === (target !== 0) && (!target || (bodies.has(target) && !ownLimb)), 'grip');
    }
    fighters.push({ consciousness, score: u8(at + 2), hands, health, stamina, grips });
  }
  return {
    tick: u32(4), applied: acknowledged ? u32(8) : null, round, number,
    arenaTick: u32(gameplay + 20), phase, winner: winner === 255 ? null : winner,
    remaining: u16(gameplay + 4), bodies, fighters
  };
}
