const sampleRate = 16000;
const buffers = new WeakMap<BaseAudioContext, AudioBuffer>();

function windBuffer(context: BaseAudioContext) {
  const cached = buffers.get(context);
  if (cached) return cached;

  const length = 4 * sampleRate;
  const overlap = sampleRate / 4;
  const buffer = context.createBuffer(1, length, sampleRate);
  const output = buffer.getChannelData(0);
  const tail = new Float32Array(overlap);
  let seed = 417;
  let low = 0;
  let rumble = 0;
  for (let i = 0; i < length + overlap; i++) {
    seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0;
    low += 0.17 * (seed / 2147483648 - 1 - low);
    rumble += 0.018 * (low - rumble);
    const phase = 2 * Math.PI * i / length;
    const swell = 0.72 + 0.18 * Math.sin(phase) + 0.10 * Math.sin(3 * phase);
    const sample = swell * (0.075 * low + 0.098 * rumble);
    if (i < length) output[i] = sample;
    else tail[i - length] = sample;
  }
  // Blend the filter's continuing output into the beginning so the loop crosses its seam at full level.
  for (let i = 0; i < overlap; i++) {
    const blend = i / (overlap - 1);
    output[i] = tail[i]! * (1 - blend) + output[i]! * blend;
  }
  buffers.set(context, buffer);
  return buffer;
}

export function playCrosswind(context: BaseAudioContext, effects: AudioNode) {
  const voice = context.createBufferSource();
  const envelope = context.createGain();
  envelope.gain.setValueAtTime(0, context.currentTime);
  envelope.gain.setTargetAtTime(1, context.currentTime, 0.09);
  voice.buffer = windBuffer(context);
  voice.loop = true;
  voice.connect(envelope).connect(effects);
  const disconnect = () => { voice.disconnect(); envelope.disconnect(); };
  voice.onended = disconnect;
  voice.start();
  return () => {
    if (context.state === 'running') {
      envelope.gain.setTargetAtTime(0, context.currentTime, 0.008);
      voice.stop(context.currentTime + 0.05);
    } else {
      voice.stop();
      disconnect();
    }
  };
}
