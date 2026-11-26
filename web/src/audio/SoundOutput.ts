import { playBreakaway } from './breakawaySound';
import { playFootstep } from './footstepSound';
import { playLanding } from './landingSound';
import { playGrip } from './gripSound';
import { playImpact } from './impactSound';
import { MatchMix } from './MatchMix';
import type { VolumeSettings } from './volumeSettings';
import type { Cues } from './MatchCues';
import type { RoundCue } from './RoundCues';
import { playMatchTone } from './matchTone';

export class SoundOutput {
  private context?: AudioContext;
  private voices: Partial<Record<keyof Cues | 'round', () => void>> = {};
  private mix?: MatchMix;
  private wantsMusic = false;
  private wantsWind = false;
  constructor(private levels: VolumeSettings) {}

  setLevels(levels: VolumeSettings) {
    this.levels = levels;
    this.mix?.setLevels(levels);
    if (levels.muted || !levels.master || !levels.effects) this.silence();
    this.refreshLoops();
  }
  setRoundAudio(playing: boolean, wind: boolean) {
    this.wantsMusic = playing;
    this.wantsWind = playing && wind;
    this.refreshLoops();
  }
  private refreshLoops() {
    const audible = !document.hidden && this.context?.state === 'running' && !this.levels.muted && this.levels.master > 0;
    this.mix?.setMusic(audible && this.wantsMusic && this.levels.music > 0);
    this.mix?.setWind(audible && this.wantsWind && this.levels.effects > 0);
  }
  visibility() {
    if (document.hidden) this.silence();
    else if (this.context) this.unlock();
    this.refreshLoops();
  }

  unlock() {
    if (this.levels.muted || document.hidden) return;
    try {
      if (!this.context) {
        this.context = new AudioContext();
        this.mix = new MatchMix(this.context, this.levels);
        this.context.onstatechange = () => {
          if (this.context?.state !== 'running') this.silence();
          this.refreshLoops();
        };
      }
      if (this.context.state !== 'running') void this.context.resume().catch(() => {});
      else this.refreshLoops();
    } catch { /* Visual warnings remain available when the audio device is unavailable. */ }
  }

  private get effects(): GainNode | undefined {
    if (this.levels.muted || !this.levels.master || !this.levels.effects || document.hidden) return;
    return this.context?.state === 'running' ? this.mix?.effects : undefined;
  }
  playRound(cue: RoundCue) {
    this.silence();
    const effects = this.effects;
    if (effects) this.voices.round = playMatchTone(effects.context, cue, effects);
  }
  play(cues: Cues) {
    const effects = this.effects;
    if (!effects) return;
    this.voices.footstep?.();
    if (cues.footstep) {
      this.voices.footstep = playFootstep(effects.context, cues.footstep, effects);
      return;
    }
    if (cues.breakaway) {
      this.voices.breakaway?.();
      this.voices.breakaway = playBreakaway(effects.context, cues.breakaway, effects);
    }
    if (cues.warning || cues.condition || cues.impact) this.voices.impact?.();
    this.voices.landing?.();
    if (cues.landing) this.voices.landing = playLanding(effects.context, cues.landing, effects);
    else this.voices.grip?.();
    if (cues.grip) this.voices.grip = playGrip(effects.context, cues.grip, effects);
    if (cues.impact) this.voices.impact = playImpact(effects.context, cues.impact, effects);
    for (const category of ['warning','condition'] as const) {
      const cue = cues[category];
      if (cue) {
        this.voices[category]?.();
        this.voices[category] = playMatchTone(effects.context, cue, effects);
      }
    }
  }
  silence() { for (const stop of Object.values(this.voices)) stop(); this.voices = {}; }
  close() {
    this.silence();
    this.mix?.close();
    this.mix = undefined;
    if (this.context) {
      this.context.onstatechange = null;
      void this.context.close().catch(() => {});
      this.context = undefined;
    }
  }
}
