export function compactRotation(data: DataView, offset: number): [number, number, number, number] {
  const omitted = data.getUint8(offset);
  if (omitted > 3) throw new RangeError('Invalid compact rotation index');
  const q: [number, number, number, number] = [0, 0, 0, 0];
  let sum = 0, largest = 0, cursor = offset + 1;
  for (let i = 0; i < 4; i++) {
    if (i === omitted) continue;
    const code = data.getInt16(cursor, true);
    if (code === -32768) throw new RangeError('Reserved compact rotation component');
    const v = code / 32767;
    q[i] = v;
    sum += v * v;
    largest = Math.max(largest, Math.abs(v));
    cursor += 2;
  }
  // Allow three quantization steps for a near tie after rounding and square-root reconstruction.
  const missing = Math.sqrt(1 - sum);
  if (!(missing + 3 / 32767 >= largest)) throw new RangeError('Invalid compact rotation magnitude');
  q[omitted] = missing;
  return q;
}
