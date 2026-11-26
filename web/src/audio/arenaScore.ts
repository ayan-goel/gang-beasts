// Eight bars at 120 BPM. Fixed-rate mono PCM keeps the same score at every device rate.
export const scoreSampleRate = 16000;

const bassRoots = [38, 38, 41, 36, 43, 41, 36, 33];
const phrases = [
  [62, 0, 65, 69], [67, 65, 0, 62], [65, 69, 72, 0], [67, 64, 0, 60],
  [70, 69, 67, 74], [72, 0, 69, 65], [67, 64, 62, 0], [61, 64, 69, 0],
];

export function renderArenaScore(): Float32Array<ArrayBuffer> {
  const samples = new Float32Array(scoreSampleRate * 16);
  function pluck(beat: number, midi: number, seconds: number, gain: number, harmonic: number) {
    const begin = Math.round(beat * 0.5 * scoreSampleRate);
    const length = Math.round(seconds * scoreSampleRate);
    const step = 2 * Math.PI * 440 * 2 ** ((midi - 69) / 12) / scoreSampleRate;
    for (let i = 0; i < length; i++) {
      const decay = 1 - i / length;
      const envelope = Math.min(1, i / 100) * decay * decay;
      const phase = step * i;
      samples[begin + i]! += gain * envelope * (Math.sin(phase) + harmonic * Math.sin(2 * phase));
    }
  }
  for (const [bar, root] of bassRoots.entries()) {
    const beat = bar * 4;
    pluck(beat, root, 0.44, 0.055, 0.22);
    pluck(beat + 1.5, root + 12, 0.18, 0.031, 0.22);
    pluck(beat + 2, root + 7, 0.36, 0.047, 0.22);
    pluck(beat + 3.25, root, 0.24, 0.038, 0.22);
    for (const [index, note] of phrases[bar]!.entries()) {
      if (note) pluck(beat + [0.5, 1.25, 2.5, 3.5][index]!, note, 0.2, 0.024, 0.42);
    }
  }
  return samples;
}
