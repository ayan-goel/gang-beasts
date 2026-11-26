import type { Snapshot } from '../net/snapshot';

const tickMs = 1000 / 60;
const maxPrediction = 6;
interface Entry { at: number; received: number; frame: Snapshot; cuts: Set<number> }
export type StreamStatus = 'buffering' | 'live' | 'estimating' | 'stalled';
export interface PoseSample {
  state: Snapshot;
  from: Snapshot;
  to: Snapshot;
  cuts: ReadonlySet<number>;
  factor: number;
  predicted: boolean;
  status: StreamStatus;
}

export function groupOf(id: number) { return id >= 65536 ? id : Math.floor((id - 1) / 16); }

function discontinuities(previous: Snapshot, next: Snapshot, spacing: number) {
  const cuts = new Set<number>();
  const distance = Math.max(1.5, spacing * .4);
  for (const [id, pose] of next.bodies) {
    const before = previous.bodies.get(id);
    if (!before) continue;
    const [x,y,z] = pose.position;
    const [px,py,pz] = before.position;
    if ((x-px)**2 + (y-py)**2 + (z-pz)**2 > distance**2) cuts.add(groupOf(id));
  }
  return cuts;
}

export class SnapshotTimeline {
  private entries: Entry[] = [];
  private rendered = 0;
  private clockOriginMs = 0;
  delayTicks = 5;
  get size() { return this.entries.length; }

  push(frame: Snapshot, received: number): boolean {
    const last = this.entries.at(-1);
    const spacing = last ? (frame.tick - last.frame.tick) >>> 0 : 0;
    if (last && (frame.round < last.frame.round || !spacing || spacing >= 0x80000000 || received < last.received)) return false;
    const at = last ? last.at + spacing : 0;
    if (!last || last.frame.round !== frame.round || spacing > 12 || received - last.received > 250) {
      this.entries = [{frame, received, at, cuts: new Set()}];
      this.rendered = at;
      this.clockOriginMs = received - at * tickMs;
      this.delayTicks = 5;
      return true;
    }
    this.entries.push({frame, at, received, cuts: discontinuities(last.frame, frame, spacing)});
    if (this.entries.length > 32) this.entries.shift();
    this.updateClock();
    return true;
  }

  private updateClock() {
    let earliestOriginMs = Infinity, latestGapEndMs = -Infinity;
    for (const [i, entry] of this.entries.entries()) {
      earliestOriginMs = Math.min(earliestOriginMs, entry.received - entry.at * tickMs);
      // Until this publication arrived the preceding pose was the newest endpoint available to draw.
      if (i) latestGapEndMs = Math.max(latestGapEndMs, entry.received - this.entries[i-1]!.at * tickMs);
    }
    const correction = Math.max(-.1 * tickMs, Math.min(.1 * tickMs, earliestOriginMs - this.clockOriginMs));
    this.clockOriginMs += correction;
    if (this.entries.length < 16) return;
    // While a slower route moves the reference forward, the current clock still needs the extra buffered time.
    const coverage = (latestGapEndMs - Math.min(this.clockOriginMs, earliestOriginMs)) / tickMs;
    const target = Math.min(6, Math.max(2.5, coverage + .5));
    this.delayTicks += Math.max(-.08, Math.min(.08, target - this.delayTicks));
  }

  sample(now: number): PoseSample | null {
    const last = this.entries.at(-1);
    if (!last) return null;
    const requested = (now - this.clockOriginMs) / tickMs - this.delayTicks;
    const target = this.rendered = Math.max(this.rendered, Math.min(last.at + maxPrediction, requested));
    let from = this.entries[0]!, to = from;
    let predicted = false;
    let status: StreamStatus = 'live';
    if (target > last.at) {
      from = this.entries.at(-2) ?? last;
      to = last;
      predicted = true;
      status = requested >= last.at + maxPrediction ? 'stalled' : 'estimating';
    } else {
      // The same bracket drives every limb, prop and discrete gameplay field in this render.
      for (const entry of this.entries) {
        if (entry.at > target + 1e-7) { to = entry; break; }
        from = to = entry;
      }
      if (requested < this.entries[0]!.at) status = 'buffering';
    }
    const factor = from === to ? 0 : Math.max(0, (target - from.at) / (to.at - from.at));
    return {state: predicted ? to.frame : from.frame, from: from.frame, to: to.frame,
      cuts: to.cuts, factor, predicted, status};
  }
}
