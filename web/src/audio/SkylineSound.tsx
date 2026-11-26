import { createContext, useContext, useEffect, useRef, useState, type ReactNode } from 'react';
import type { MatchView } from '../rendering/SkylineView';
import { HazardWarnings, type AudibleHazard } from './HazardWarnings';
import { playWarningTone } from './warningTone';
import './sound.css';

const preference = 'skyline.audio.v1.muted';
class SoundOutput {
  private context?: AudioContext;
  private stopVoice?: () => void;
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
  play(hazard: AudibleHazard) {
    if (this.muted || this.context?.state !== 'running' || document.hidden) return;
    this.silence();
    this.stopVoice = playWarningTone(this.context, hazard);
  }
  silence() { this.stopVoice?.(); this.stopVoice = undefined; }
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

export function useHazardSound(match: MatchView | undefined, active: boolean) {
  const output = useContext(AudioSettings)?.output;
  const cursor = useRef(new HazardWarnings());
  useEffect(() => {
    const reset = () => { cursor.current.reset(); output?.silence(); };
    document.addEventListener('visibilitychange', reset);
    return () => { document.removeEventListener('visibilitychange', reset); reset(); };
  }, [output]);
  useEffect(() => {
    if (!active || !match || match.phase !== 'playing' || document.hidden) {
      cursor.current.reset();
      output?.silence();
      return;
    }
    const cue = cursor.current.observe(match.epoch, match.hazard, performance.now());
    if (cue) output?.play(cue);
  }, [match, active, output]);
}
