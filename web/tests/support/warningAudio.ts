import type { Page } from '@playwright/test';

interface WarningSample { wave: OscillatorType; state: AudioContextState }
declare global {
  interface Window { skylineAudioProbe: {contexts: number; warnings: WarningSample[]; impacts: number} }
}

export async function observeWarningAudio(page: Page) {
  await page.addInitScript(() => {
    const probe = window.skylineAudioProbe = {contexts:0, impacts:0, warnings:[] as WarningSample[]};
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
        voice.start = when => {
          probe.warnings.push({wave:voice.type, state:this.state});
          start(when);
        };
        return voice;
      }
    };
  });
}
export const warningSamples = (page: Page) => page.evaluate(() => window.skylineAudioProbe.warnings);

export const impactCount = (page: Page) => page.evaluate(() => window.skylineAudioProbe.impacts);
