export function summarize(values) {
  if (!values.length) throw new Error('No frames measured');
  const sorted = [...values].sort((a, b) => a - b);
  // Nearest rank keeps a missed refresh in the tail even for short diagnostic windows.
  const rank = p => Number(sorted[Math.ceil(p * sorted.length) - 1].toFixed(3));
  return {count:sorted.length, p50:rank(.5), p95:rank(.95), p99:rank(.99), max:rank(1)};
}

export function splitCapture(capture) {
  const identity = capture.subarray(0, 128).toString('utf8');
  if (!/^[a-f0-9]{128}$/.test(identity)) throw new Error('Invalid engine or map hash');
  return {identity, bytes:capture.subarray(128)};
}

export function unpack(bytes) {
  const frames = [];
  let previous;
  for (let cursor = 0; cursor < bytes.length;) {
    const size = bytes.readUInt32LE(cursor);
    cursor += 4;
    const record = bytes.subarray(cursor, cursor + size);
    if (size < 8 || record.length !== size || record[0] !== 2) throw new Error('Invalid capture record');
    const tick = record.readUInt32LE(4);
    if (previous !== undefined && tick !== previous + 2) throw new Error('Missing capture tick');
    previous = tick;
    frames.push(record);
    cursor += size;
  }
  if (!frames.length) throw new Error('Empty capture');
  return frames;
}
