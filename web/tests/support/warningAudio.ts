import type { Page } from '@playwright/test';

interface WarningSample { wave: OscillatorType; state: AudioContextState }
declare global {
  interface Window { skylineAudioProbe: {contexts: number; warnings: WarningSample[]; impacts: number; landings: number; grips: number[]; rounds: number[]} }
}

export async function observeWarningAudio(page: Page) {
  await page.addInitScript(() => {
    const probe = window.skylineAudioProbe = {contexts:0, impacts:0, landings:0, grips:[] as number[], rounds:[] as number[], warnings:[] as WarningSample[]};
    window.AudioContext = class extends AudioContext {
      constructor(options?: AudioContextOptions) { super(options); probe.contexts++; }
      override createBufferSource() {
        const voice = super.createBufferSource();
        const start = voice.start.bind(voice);
        voice.start = (...args) => {
          if (this.state === 'running' && voice.buffer) probe.impacts++;
          start(...args);
        };
        return voice;
      }
      override createOscillator() {
        const voice = super.createOscillator();
        const start = voice.start.bind(voice);
        let pitch = 0;
        const set = voice.frequency.setValueAtTime.bind(voice.frequency);
        voice.frequency.setValueAtTime = (hz, time) => { pitch = hz; return set(hz, time); };
        voice.start = when => {
          if (voice.type === 'sine' && [440,660,554,294,784].includes(pitch)) {
            if (this.state === 'running') probe.rounds.push(pitch);
          } else if (voice.type === 'sawtooth') {
            if (this.state === 'running') probe.grips.push(pitch);
          } else if (voice.type === 'triangle' && pitch >= 85 && pitch <= 105) {
            if (this.state === 'running') probe.landings++;
          } else probe.warnings.push({wave:voice.type, state:this.state});
          start(when);
        };
        return voice;
      }
    };
  });
}
export const warningSamples = (page: Page) => page.evaluate(() => window.skylineAudioProbe.warnings);

export const impactCount = (page: Page) => page.evaluate(() => window.skylineAudioProbe.impacts);

export const gripPitches = (page: Page) => page.evaluate(() => window.skylineAudioProbe.grips);

export const landingCount = (page: Page) => page.evaluate(() => window.skylineAudioProbe.landings);
export const roundPitches = (page: Page) => page.evaluate(() => window.skylineAudioProbe.rounds);
