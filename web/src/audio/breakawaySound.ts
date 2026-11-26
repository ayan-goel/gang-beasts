import type { SkylineContent } from '../lobby/content';
import { playSpatialBuffer, type CameraPosition } from './spatialMix';

export function recentBreakawayMount(map: SkylineContent['map'], tick: number): [number, number, number] | undefined {
  const { west_tick, east_tick } = map.schedule;
  const west = tick < east_tick;
  const age = tick - (west ? west_tick : east_tick);
  if (age < 0 || age > 12) return;
  const panel = map.objects.find(obj => obj.id === (west ? 8 : 9));
  if (!panel) return;
  const [x,y,z] = panel.center;
  return [x - Math.sign(x) * panel.half[0], y + panel.half[1], z];
}

const buffers = new WeakMap<BaseAudioContext, AudioBuffer>();

function panelBuffer(context: BaseAudioContext) {
  const existing = buffers.get(context);
  if (existing) return existing;
  const rate = context.sampleRate;
  const buffer = context.createBuffer(1, Math.ceil(.32 * rate), rate);
  const samples = buffer.getChannelData(0);
  let seed = 101;
  for (let i = 0; i < samples.length; i++) {
    const t = i / rate;
    seed = (Math.imul(seed,1664525) + 1013904223) >>> 0;
    const crack = .37 * (seed / 2147483648 - 1) * Math.exp(-110*t);
    const ring = .31 * Math.sin(2*Math.PI*218*t) * Math.exp(-16*t)
      + .20 * Math.sin(2*Math.PI*577*t) * Math.exp(-25*t)
      + .12 * Math.sin(2*Math.PI*1310*t) * Math.exp(-41*t);
    samples[i] = (crack + ring) * Math.min(1,t/.002,(samples.length-1-i)/(.012*rate));
  }
  buffers.set(context,buffer);
  return buffer;
}

export function playBreakaway(context: BaseAudioContext, source: CameraPosition, destination: AudioNode = context.destination) {
  return playSpatialBuffer(context,panelBuffer(context),1,.11,source,destination);
}
