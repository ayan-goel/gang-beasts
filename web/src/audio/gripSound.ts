import { distanceGain, type CameraPosition } from './spatialMix';

export type GripKind = 'catch' | 'strain' | 'release';
export interface GripCue extends CameraPosition { kind: GripKind }

const sounds = {
  catch: {from:320,to:115,length:0.055,peak:0.065},
  strain: {from:145,to:72,length:0.19,peak:0.075},
  release: {from:180,to:360,length:0.07,peak:0.045},
};

export function playGrip(context: BaseAudioContext, cue: GripCue) {
  const sound = sounds[cue.kind];
  const voice = context.createOscillator();
  const filter = context.createBiquadFilter();
  const gain = context.createGain();
  const pan = context.createStereoPanner();
  const start = context.currentTime, end = start + sound.length;
  voice.type = 'sawtooth';
  voice.frequency.setValueAtTime(sound.from,start);
  voice.frequency.exponentialRampToValueAtTime(sound.to,end);
  filter.type = 'lowpass';
  filter.frequency.value = 950;
  filter.Q.value = 0.5;
  gain.gain.setValueAtTime(0,start);
  gain.gain.linearRampToValueAtTime(sound.peak * distanceGain(cue.distance),start + 0.009);
  gain.gain.exponentialRampToValueAtTime(0.0001,end);
  gain.gain.linearRampToValueAtTime(0,end + 0.005);
  pan.pan.value = cue.pan;
  voice.connect(filter).connect(gain).connect(pan).connect(context.destination);
  const disconnect = () => { voice.disconnect(); filter.disconnect(); gain.disconnect(); pan.disconnect(); };
  voice.onended = disconnect;
  voice.start(start);
  voice.stop(end + 0.006);
  return () => { voice.stop(); disconnect(); };
}
