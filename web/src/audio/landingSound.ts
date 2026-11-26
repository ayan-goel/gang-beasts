import { distanceGain, type CameraPosition } from './spatialMix';

export interface LandingEvent { tick: number; impulse: number }
export interface LandingCue extends CameraPosition { impulse: number }

export function playLanding(context: BaseAudioContext, cue: LandingCue, destination: AudioNode = context.destination) {
  const voice = context.createOscillator();
  const gain = context.createGain();
  const pan = context.createStereoPanner();
  const start = context.currentTime;
  const strength = Math.min(1, cue.impulse / 90);
  const end = start + 0.09 + 0.07 * strength;
  const attenuation = distanceGain(cue.distance);
  voice.type = 'triangle';
  voice.frequency.setValueAtTime(105 - 20 * strength, start);
  voice.frequency.exponentialRampToValueAtTime(39, end);
  gain.gain.setValueAtTime(0, start);
  gain.gain.linearRampToValueAtTime((0.05 + 0.075 * strength) * attenuation, start + 0.004);
  gain.gain.exponentialRampToValueAtTime(0.0001 * attenuation, end);
  gain.gain.linearRampToValueAtTime(0, end + 0.004);
  pan.pan.value = cue.pan;
  voice.connect(gain).connect(pan).connect(destination);
  const disconnect = () => { voice.disconnect(); gain.disconnect(); pan.disconnect(); };
  voice.onended = disconnect;
  voice.start(start);
  voice.stop(end + 0.005);
  return () => { voice.stop(); disconnect(); };
}
