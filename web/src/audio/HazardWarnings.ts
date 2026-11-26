import type { MatchView } from '../rendering/SkylineView';

type Hazard = MatchView['hazard'];
export type AudibleHazard = Exclude<Hazard, 'quiet'>;
const stages: readonly Hazard[] = ['quiet', 'west_warning', 'east_warning', 'gust_warning', 'gust'];

export class HazardWarnings {
  private epoch?: string;
  private stage = 0;
  private observedAt = 0;

  reset() { this.epoch = undefined; }

  observe(epoch: string, hazard: Hazard, now: number): AudibleHazard | undefined {
    const stage = stages.indexOf(hazard);
    // A full snapshot after a rendering or network gap describes the present, not a new warning.
    const continuous = epoch === this.epoch && now - this.observedAt <= 500;
    const fresh = continuous && stage > this.stage;
    this.stage = epoch === this.epoch ? Math.max(this.stage, stage) : stage;
    this.epoch = epoch;
    this.observedAt = now;
    return fresh && hazard !== 'quiet' ? hazard : undefined;
  }
}
