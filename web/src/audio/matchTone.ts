import type { AudibleHazard, ConditionCue } from './MatchCues';
import type { RoundCue } from './RoundCues';

interface Tone { wave: OscillatorType; from: number; to: number; pulses: number; length: number }
const roundTones: Record<RoundCue, Tone> = {
  count: {wave:'sine', from:440, to:440, pulses:1, length:0.075},
  go: {wave:'sine', from:660, to:880, pulses:1, length:0.16},
  round_win: {wave:'sine', from:554, to:740, pulses:2, length:0.12},
  draw: {wave:'sine', from:294, to:247, pulses:2, length:0.13},
  match_win: {wave:'sine', from:784, to:1047, pulses:3, length:0.16},
};
const tones: Record<AudibleHazard | ConditionCue | RoundCue, Tone> = {
  ...roundTones,
  knocked_out: {wave:'triangle', from:175, to:58, pulses:1, length:0.31},
  recovering: {wave:'sine', from:310, to:545, pulses:2, length:0.11},
  west_warning: {wave:'triangle', from:520, to:370, pulses:2, length:0.16},
  east_warning: {wave:'triangle', from:680, to:470, pulses:3, length:0.13},
  gust_warning: {wave:'sine', from:215, to:620, pulses:2, length:0.28},
  gust: {wave:'sine', from:430, to:145, pulses:1, length:0.48},
};

export function playMatchTone(context: BaseAudioContext, cue: AudibleHazard | ConditionCue | RoundCue) {
  const tone = tones[cue];
  const peak = cue in roundTones ? 0.07 : cue === 'knocked_out' || cue === 'recovering' ? 0.08 : 0.12;
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
    volume.gain.linearRampToValueAtTime(peak, at + 0.016);
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
