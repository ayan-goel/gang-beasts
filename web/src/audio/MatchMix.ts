import { playCrosswind } from './crosswindSound';
import { renderArenaScore, scoreSampleRate } from './arenaScore';
import type { VolumeSettings } from './volumeSettings';

export class MatchMix {
  private readonly master: GainNode;
  readonly effects: GainNode;
  readonly music: GainNode;
  private stopWind?: () => void;
  private score?: AudioBuffer;
  private loop?: AudioBufferSourceNode;

  constructor(private context: BaseAudioContext, levels: VolumeSettings) {
    this.master = context.createGain();
    this.music = context.createGain();
    this.effects = context.createGain();
    this.music.connect(this.master);
    this.effects.connect(this.master);
    this.master.connect(context.destination);
    this.setLevels(levels, true);
  }

  setLevels(levels: VolumeSettings, initial = false) {
    const set = (node: GainNode, percent: number) => {
      const gain = percent / 100;
      if (initial) node.gain.value = gain;
      else node.gain.setTargetAtTime(gain, this.context.currentTime, 0.008);
    };
    set(this.master, levels.muted ? 0 : levels.master);
    set(this.music, levels.music);
    set(this.effects, levels.effects);
  }

  setMusic(active: boolean) {
    if (active === !!this.loop) return;
    if (!active) {
      this.loop!.stop();
      this.loop!.disconnect();
      this.loop = undefined;
      return;
    }
    if (!this.score) {
      const samples = renderArenaScore();
      this.score = this.context.createBuffer(1, samples.length, scoreSampleRate);
      this.score.copyToChannel(samples, 0);
    }
    const voice = this.context.createBufferSource();
    voice.buffer = this.score;
    voice.loop = true;
    voice.connect(this.music);
    voice.start();
    this.loop = voice;
  }

  setWind(active: boolean) {
    if (active === !!this.stopWind) return;
    if (active) this.stopWind = playCrosswind(this.context, this.effects);
    else {
      this.stopWind?.();
      this.stopWind = undefined;
    }
  }

  close() {
    this.setWind(false);
    this.setMusic(false);
    this.effects.disconnect();
    this.music.disconnect();
    this.master.disconnect();
  }
}
