import { playLanding } from './landingSound';
import { playGrip } from './gripSound';
import type { GripAudioState } from './gripCues';
import { playImpact, type ImpactSource } from './impactSound';
import { createContext, useContext, useEffect, useRef, useState, type ReactNode } from 'react';
import type { MatchView } from '../rendering/SkylineView';
import type { FighterPhase } from '../game/fighterCondition';
import { MatchCues, type Cues } from './MatchCues';
import { RoundCues, type RoundCue } from './RoundCues';
import { playMatchTone } from './matchTone';
import './sound.css';

const preference = 'skyline.audio.v1.muted';
class SoundOutput {
  private context?: AudioContext;
  private voices: Partial<Record<keyof Cues | 'round', () => void>> = {};
  muted = false;

  unlock() {
    if (this.muted || document.hidden) return;
    try {
      if (!this.context) {
        this.context = new AudioContext();
        this.context.onstatechange = () => { if (this.context?.state !== 'running') this.silence(); };
      }
      if (this.context.state !== 'running') void this.context.resume().catch(() => {});
    } catch { /* Visual warnings remain available when the audio device is unavailable. */ }
  }
  resume() { if (this.context) this.unlock(); }
  playRound(cue: RoundCue) {
    this.silence();
    if (!this.muted && this.context?.state === 'running' && !document.hidden) {
      this.voices.round = playMatchTone(this.context, cue);
    }
  }
  play(cues: Cues) {
    if (this.muted || this.context?.state !== 'running' || document.hidden) return;
    if (cues.warning || cues.condition || cues.impact) this.voices.impact?.();
    this.voices.landing?.();
    if (cues.landing) this.voices.landing = playLanding(this.context, cues.landing);
    else this.voices.grip?.();
    if (cues.grip) this.voices.grip = playGrip(this.context, cues.grip);
    if (cues.impact) this.voices.impact = playImpact(this.context, cues.impact);
    for (const category of ['warning','condition'] as const) {
      const cue = cues[category];
      if (cue) {
        this.voices[category]?.();
        this.voices[category] = playMatchTone(this.context, cue);
      }
    }
  }
  silence() { for (const stop of Object.values(this.voices)) stop(); this.voices = {}; }
  close() {
    this.silence();
    if (this.context) {
      this.context.onstatechange = null;
      void this.context.close().catch(() => {});
      this.context = undefined;
    }
  }
}

const AudioSettings = createContext<{output: SoundOutput; muted: boolean; toggle: () => void} | null>(null);
export function SkylineSound({children}: {children: ReactNode}) {
  const [output] = useState(() => new SoundOutput());
  const [muted, setMuted] = useState(() => {
    try { return localStorage.getItem(preference) === 'true'; } catch { return false; }
  });
  useEffect(() => {
    output.muted = muted;
    if (muted) output.silence();
    const interact = (event: Event) => { if (event.isTrusted) output.unlock(); };
    const visibility = () => { if (document.hidden) output.silence(); else output.resume(); };
    window.addEventListener('click', interact, true);
    window.addEventListener('keydown', interact, true);
    document.addEventListener('visibilitychange', visibility);
    return () => {
      window.removeEventListener('click', interact, true);
      window.removeEventListener('keydown', interact, true);
      document.removeEventListener('visibilitychange', visibility);
    };
  }, [output, muted]);
  useEffect(() => () => output.close(), [output]);
  const toggle = () => {
    output.muted = !muted;
    if (output.muted) output.silence(); else output.unlock();
    setMuted(output.muted);
    try { localStorage.setItem(preference, String(output.muted)); } catch { /* Keep the tab preference in memory. */ }
  };
  return <AudioSettings.Provider value={{output, muted, toggle}}>{children}</AudioSettings.Provider>;
}

export function MuteButton() {
  const settings = useContext(AudioSettings);
  return settings && <button className="skyline-mute" aria-label="Mute game audio" aria-pressed={settings.muted}
    onClick={settings.toggle}>{settings.muted ? 'Sound off' : 'Sound on'}</button>;
}

export function useMatchSound(match: MatchView | undefined, active: boolean, phases: readonly FighterPhase[], audio?: GripAudioState & {impacts?: readonly ImpactSource[]}) {
  const output = useContext(AudioSettings)?.output;
  const cursor = useRef(new MatchCues());
  const rounds = useRef(new RoundCues());
  useEffect(() => {
    const reset = () => { cursor.current.reset(); rounds.current.reset(); output?.silence(); };
    document.addEventListener('visibilitychange', reset);
    return () => { document.removeEventListener('visibilitychange', reset); reset(); };
  }, [output]);
  useEffect(() => {
    if (!active || !match || document.hidden) {
      cursor.current.reset();
      rounds.current.reset();
      output?.silence();
      return;
    }
    const now = performance.now();
    const round = rounds.current.observe(match, now);
    if (round) output?.playRound(round);
    if (match.phase !== 'playing') {
      cursor.current.reset();
      return;
    }
    const cues = cursor.current.observe(match.epoch, match.hazard, phases, now, audio?.impacts, audio);
    if (cues.warning || cues.condition || cues.impact || cues.grip || cues.landing) output?.play(cues);
  }, [match, active, output, phases, audio]);
}
