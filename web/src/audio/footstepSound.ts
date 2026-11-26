import { playSpatialBuffer, type CameraPosition } from './spatialMix';

export interface FootstepCue extends CameraPosition { tick: number }

const buffers = new WeakMap<BaseAudioContext,AudioBuffer>();

function soleBuffer(context: BaseAudioContext) {
  const existing = buffers.get(context);
  if (existing) return existing;
  const rate = context.sampleRate;
  const buffer = context.createBuffer(1,Math.ceil(.08 * rate),rate);
  const samples = buffer.getChannelData(0);
  let seed = 43, surface = 0;
  const smoothing = 1 - Math.exp(-2 * Math.PI * 900 / rate);
  for (let i = 0; i < samples.length; i++) {
    const t = i / rate;
    seed = (Math.imul(seed,1664525) + 1013904223) >>> 0;
    surface += smoothing * ((seed / 2147483648 - 1) - surface);
    const heel = Math.sin(2 * Math.PI * 125 * t) * Math.exp(-95 * t);
    const scuff = surface * Math.exp(-55 * t);
    samples[i] = (.68 * heel + .32 * scuff) * Math.min(1, t / .002, (samples.length - 1 - i) / (.006 * rate));
  }
  buffers.set(context,buffer);
  return buffer;
}

export function playFootstep(context: BaseAudioContext, cue: FootstepCue, destination: AudioNode = context.destination) {
  return playSpatialBuffer(context, soleBuffer(context), .96 + .02 * (cue.tick % 5), .06, cue, destination);
}
