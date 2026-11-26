import type { AudibleHazard } from './HazardWarnings';

interface Tone { wave: OscillatorType; from: number; to: number; pulses: number; length: number }
const tones: Record<AudibleHazard, Tone> = {
  west_warning: {wave:'triangle', from:520, to:370, pulses:2, length:0.16},
  east_warning: {wave:'triangle', from:680, to:470, pulses:3, length:0.13},
  gust_warning: {wave:'sine', from:215, to:620, pulses:2, length:0.28},
  gust: {wave:'sine', from:430, to:145, pulses:1, length:0.48},
};

export function playWarningTone(context: BaseAudioContext, hazard: AudibleHazard) {
  const tone = tones[hazard];
  const voice = context.createOscillator();
  const volume = context.createGain();
  const start = context.currentTime;
  voice.type = tone.wave;
  volume.gain.value = 0;
  for (let i = 0; i < tone.pulses; i++) {
    const at = start + i * (tone.length + 0.07);
    voice.frequency.setValueAtTime(tone.from, at);
    voice.frequency.exponentialRampToValueAtTime(tone.to, at + tone.length);
    volume.gain.setValueAtTime(0, at);
    volume.gain.linearRampToValueAtTime(0.12, at + 0.016);
    volume.gain.exponentialRampToValueAtTime(0.001, at + tone.length);
    volume.gain.linearRampToValueAtTime(0, at + tone.length + 0.006);
  }
  voice.connect(volume).connect(context.destination);
  const disconnect = () => { voice.disconnect(); volume.disconnect(); };
  voice.onended = disconnect;
  voice.start(start);
  voice.stop(start + tone.pulses * (tone.length + 0.07));
  return () => { voice.stop(); disconnect(); };
}
