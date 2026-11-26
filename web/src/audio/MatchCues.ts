import type { Impact, ImpactSource } from './impactSound';
import type { MatchView } from '../rendering/SkylineView';
import type { FighterPhase } from '../game/fighterCondition';

type Hazard = MatchView['hazard'];
export type AudibleHazard = Exclude<Hazard, 'quiet'>;
export type ConditionCue = 'knocked_out' | 'recovering';
export interface Cues { warning?: AudibleHazard; condition?: ConditionCue; impact?: Impact }
const stages: readonly Hazard[] = ['quiet', 'west_warning', 'east_warning', 'gust_warning', 'gust'];

export class MatchCues {
  private epoch?: string;
  private stage = 0;
  private observedAt = 0;
  private phases: readonly FighterPhase[] = [];

  private sources: readonly ImpactSource[] = [];
  private lastImpact: number[] = [];

  reset() { this.epoch = undefined; }

  observe(epoch: string, hazard: Hazard, phases: readonly FighterPhase[], now: number, sources: readonly ImpactSource[] = []): Cues {
    const stage = stages.indexOf(hazard);
    const cues: Cues = {};
    // A full snapshot after a rendering or network gap describes the present, not a new event.
    if (epoch === this.epoch && now - this.observedAt <= 500) {
      if (stage > this.stage && hazard !== 'quiet') cues.warning = hazard;
      if (phases.some((phase, i) => phase === 'knocked_out' && this.phases[i] === 'active')) {
        cues.condition = 'knocked_out';
      } else if (phases.some((phase, i) => phase === 'recovering' && this.phases[i] === 'knocked_out')) {
        cues.condition = 'recovering';
      }
      for (const [slot, source] of sources.entries()) {
        const prior = this.sources[slot];
        if (!prior || phases[slot] !== 'active' || this.phases[slot] !== 'active') continue;
        const damage = prior.health - source.health;
        if (damage < 1 || now - (this.lastImpact[slot] ?? -Infinity) < 120) continue;
        // Consume every qualifying contact so a simultaneous pile-up gets one voice.
        this.lastImpact[slot] = now;
        if (!cues.warning && !cues.condition && damage > (cues.impact?.damage ?? 0)) {
          cues.impact = {damage, pan:source.pan, distance:source.distance};
        }
      }
    } else {
      this.lastImpact = [];
    }
    this.sources = sources;
    this.stage = epoch === this.epoch ? Math.max(this.stage, stage) : stage;
    this.epoch = epoch;
    this.observedAt = now;
    this.phases = phases;
    return cues;
  }
}
