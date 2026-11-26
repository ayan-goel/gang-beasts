import { createContext, useContext, useEffect, useRef, useState, type ReactNode } from 'react';
import type { MatchView } from '../rendering/SkylineView';
import type { FighterPhase } from '../game/fighterCondition';
import { MatchCues } from './MatchCues';
import { RoundCues } from './RoundCues';
import { SoundOutput } from './SoundOutput';
import { readVolume, saveVolume, type VolumeSettings, type VolumeChannel } from './volumeSettings';
import type { GripAudioState } from './gripCues';
import type { ImpactSource } from './impactSound';
import './sound.css';

const AudioSettings = createContext<{
  output: SoundOutput; levels: VolumeSettings; change: (patch: Partial<VolumeSettings>) => void;
} | null>(null);

export function SkylineSound({children}: {children: ReactNode}) {
  const [levels, setLevels] = useState(() => readVolume());
  const current = useRef(levels);
  const [output] = useState(() => new SoundOutput(levels));
  useEffect(() => {
    const interact = (event: Event) => { if (event.isTrusted) output.unlock(); };
    const visibility = () => output.visibility();
    window.addEventListener('click', interact, true);
    window.addEventListener('keydown', interact, true);
    document.addEventListener('visibilitychange', visibility);
    return () => {
      window.removeEventListener('click', interact, true);
      window.removeEventListener('keydown', interact, true);
      document.removeEventListener('visibilitychange', visibility);
      output.close();
    };
  }, [output]);
  const change = (patch: Partial<VolumeSettings>) => {
    const next = {...current.current, ...patch};
    current.current = next;
    output.setLevels(next);
    if (patch.muted === false) output.unlock();
    setLevels(next);
    saveVolume(next);
  };
  return <AudioSettings.Provider value={{output, levels, change}}>{children}</AudioSettings.Provider>;
}

export function MuteButton() {
  const settings = useContext(AudioSettings);
  if (!settings) return null;
  const {muted} = settings.levels;
  return <button className="skyline-mute" aria-label="Mute game audio" aria-pressed={muted}
    onClick={() => settings.change({muted:!muted})}>{muted ? 'Sound off' : 'Sound on'}</button>;
}

const channelNames: Record<VolumeChannel, string> = {master:'Master', music:'Music', effects:'Effects'};
export function VolumeControls() {
  const settings = useContext(AudioSettings);
  return settings && <fieldset className="match-levels"><legend>Volume</legend>
    {(['master','music','effects'] as const).map(channel => <label key={channel}>
      {channelNames[channel]}<input aria-label={channelNames[channel] + ' volume'} type="range" min={0} max={100}
        value={settings.levels[channel]} aria-valuetext={settings.levels[channel] + '%'}
        onChange={event => settings.change({[channel]:event.currentTarget.valueAsNumber})} />
      <span>{settings.levels[channel]}%</span></label>)}
  </fieldset>;
}

export function useMatchSound(match: MatchView | undefined, active: boolean, phases: readonly FighterPhase[], audio?: GripAudioState & {impacts?: readonly ImpactSource[]}) {
  const output = useContext(AudioSettings)?.output;
  const playing = active && match?.phase === 'playing';
  useEffect(() => {
    output?.setMusic(playing);
    return () => output?.setMusic(false);
  }, [output, playing]);
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
    if (cues.warning || cues.condition || cues.impact || cues.grip || cues.landing || cues.footstep) output?.play(cues);
  }, [match, active, output, phases, audio]);
}
