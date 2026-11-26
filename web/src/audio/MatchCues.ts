import type { FootstepCue } from './footstepSound';
import type { LandingCue } from './landingSound';
import { changedGrip, type GripAudioState } from './gripCues';
import type { GripCue, GripKind } from './gripSound';
import type { Impact, ImpactSource } from './impactSound';
import type { MatchView } from '../rendering/SkylineView';
import type { FighterPhase } from '../game/fighterCondition';

import type { CameraPosition } from './spatialMix';

export interface MatchAudioState extends GripAudioState { impacts?: readonly ImpactSource[]; breakaway?: CameraPosition }

type Hazard = MatchView['hazard'];
export type AudibleHazard = Exclude<Hazard, 'quiet'>;
export type ConditionCue = 'knocked_out' | 'recovering';
export interface Cues { warning?: AudibleHazard; breakaway?: CameraPosition; condition?: ConditionCue; impact?: Impact; grip?: GripCue; landing?: LandingCue; footstep?: FootstepCue }
const stages: readonly Hazard[] = ['quiet', 'west_warning', 'east_warning', 'gust_warning', 'gust'];

export class MatchCues {
  private epoch?: string;
  private stage = 0;
  private observedAt = 0;
  private phases: readonly FighterPhase[] = [];

  private sources: readonly ImpactSource[] = [];
  private lastStep = -Infinity;
  private lastImpact: number[] = [];

  private grips?: GripAudioState;
  private lastGrip: Partial<Record<GripKind, number>> = {};

  reset() { this.epoch = undefined; }

  observe(epoch: string, hazard: Hazard, phases: readonly FighterPhase[], now: number, sources: readonly ImpactSource[] = [], grips?: MatchAudioState): Cues {
    const stage = stages.indexOf(hazard);
    const cues: Cues = {};
    // A full snapshot after a rendering or network gap describes the present, not a new event.
    if (epoch === this.epoch && now - this.observedAt <= 500) {
      if (stage > this.stage && hazard !== 'quiet') cues.warning = hazard;
      if (stage === this.stage + 1 && (hazard === 'east_warning' || hazard === 'gust_warning') && grips?.breakaway) {
        cues.breakaway = grips.breakaway;
      }
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
      if (this.grips && grips && !cues.condition && !cues.warning && !cues.impact) {
        const grip = changedGrip(this.grips, grips, phases, sources);
        if (grip && now - (this.lastGrip[grip.kind] ?? -Infinity) >= 150) {
          cues.grip = grip;
          this.lastGrip[grip.kind] = now;
        }
      }
      if (!cues.warning && !cues.condition && !cues.impact && !cues.grip) {
        for (const [slot, source] of sources.entries()) {
          const event = source.landing;
          const prior = this.sources[slot];
          if (event && prior && phases[slot] !== 'eliminated' && event.tick !== prior.landing?.tick
            && event.impulse > (cues.landing?.impulse ?? 0)) {
            cues.landing = {impulse:event.impulse, pan:source.pan, distance:source.distance};
          }
        }
        if (!cues.landing && now - this.lastStep >= 90) {
          for (const [slot, source] of sources.entries()) {
            const prior = this.sources[slot];
            if (source.footstep && prior && source.footstep !== prior.footstep && phases[slot] === 'active'
                && source.distance < (cues.footstep?.distance ?? Infinity)) {
              cues.footstep = {tick:source.footstep, distance:source.distance, pan:source.pan};
            }
          }
          if (cues.footstep) this.lastStep = now;
        }
      }
    } else {
      this.lastStep = -Infinity;
      this.lastGrip = {};
      this.lastImpact = [];
    }
    this.grips = grips;
    this.sources = sources;
    this.stage = epoch === this.epoch ? Math.max(this.stage, stage) : stage;
    this.epoch = epoch;
    this.observedAt = now;
    this.phases = phases;
    return cues;
  }
}
