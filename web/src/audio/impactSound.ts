import type { LandingEvent } from './landingSound';
import { playSpatialBuffer, type CameraPosition } from './spatialMix';

export interface ImpactSource extends CameraPosition { health: number; footstep?: number; landing?: LandingEvent }
export interface Impact extends CameraPosition { damage: number }

const buffers = new WeakMap<BaseAudioContext, AudioBuffer>();

function contactBuffer(context: BaseAudioContext) {
  const existing = buffers.get(context);
  if (existing) return existing;
  const buffer = context.createBuffer(1, Math.ceil(context.sampleRate * 0.12), context.sampleRate);
  const samples = buffer.getChannelData(0);
  let seed = 17, phase = 0;
  for (let i = 0; i < samples.length; i++) {
    const t = i / context.sampleRate;
    seed = (Math.imul(seed, 1664525) + 1013904223) >>> 0;
    phase += 2 * Math.PI * (65 + 100 * Math.exp(-50*t)) / context.sampleRate;
    const thump = 0.74 * Math.sin(phase) * Math.exp(-40*t);
    const tap = 0.26 * (2 * seed / 4294967296 - 1) * Math.exp(-160*t);
    samples[i] = (thump + tap) * Math.min(1, t/0.002, (samples.length-1-i)/(0.008*context.sampleRate));
  }
  buffers.set(context, buffer);
  return buffer;
}

export function playImpact(context: BaseAudioContext, impact: Impact, destination: AudioNode = context.destination) {
  const strength = Math.min(1, impact.damage / 35);
  return playSpatialBuffer(context, contactBuffer(context), 1.1 - 0.2 * strength, .07 + .09 * strength, impact, destination);
}
