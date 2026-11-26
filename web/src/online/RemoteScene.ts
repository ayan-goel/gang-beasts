import { fighterPhases, type FighterPhase } from '../game/fighterCondition';
import { writePose } from './samplePose';
import type { StreamStatus } from './SnapshotTimeline';
import { bindRemoteControls } from './RemoteControls';
import { FixtureScene } from '../rendering/FixtureScene';
import type { MatchView } from '../rendering/SkylineView';
import type { SkylineContent } from '../lobby/content';
import type { Snapshot } from '../net/snapshot';
import type { MatchFeed } from './MatchFeed';

const colors = ['#f38e66','#64b7dd','#e8bd59','#7dd3b1','#bd9ae6','#e69eb9','#bbd17e','#a7b4c9'];
const phases = ['countdown','playing','round_results','match_results'] as const;

type RemoteMatch = MatchView & {conditions: FighterPhase[]};

export function matchView(frame: Snapshot, content: SkylineContent): RemoteMatch {
  const t = frame.arenaTick;
  const s = content.map.schedule;
  return {phase: phases[frame.phase]!, round: frame.number, epoch: String(frame.round), remaining: frame.remaining,
    scores: frame.fighters.map(f => f.score), winner: frame.winner,
    conditions: frame.fighters.map(f => fighterPhases[f.consciousness]!),
    visible: frame.fighters.map(f => f.consciousness !== 3),
    timeLeft: Math.max(0, (t < s.gust_tick ? s.gust_tick : s.end_tick) - t),
    hazard: t >= s.gust_tick ? 'gust' : t >= s.east_tick ? 'gust_warning' : t >= s.west_tick ? 'east_warning' : t >= s.warn_tick ? 'west_warning' : 'quiet'};
}

export function startRemoteScene(container: HTMLElement, content: SkylineContent, feed: MatchFeed, update: (match: RemoteMatch) => void, status: (value: StreamStatus) => void) {
  const scene = new FixtureScene(container, content.fighter, {map: content.map, model: content.model, kit: feed.fighters[0]!.color, color: colors[feed.fighters[0]!.color]});
  const bodies = feed.fighters.map(() => new Float32Array(15*14));
  const objects = new Float32Array(8*content.map.objects.length);
  let animation = 0, priorTime = performance.now(), prior: Snapshot | null = null, match: RemoteMatch | null = null;
  let previousStatus: StreamStatus | undefined;
  let unbind: (() => void) | undefined;
  try {
    for (const fighter of feed.fighters.slice(1)) scene.addOpponent(content.fighter, colors[fighter.color], fighter.color);
    unbind = bindRemoteControls(container.querySelector('canvas')!, feed, scene.groupCamera!);
    function render(now: number) {
      const sample = feed.timeline.sample(now);
      if (sample) {
        for (const [slot, buffer] of bodies.entries()) {
          for (let i = 0; i < 15; i++) writePose(sample, 1 + slot*16 + i, buffer, i*14);
        }
        content.map.objects.forEach(({id}, i) => {
          objects[i*8+7] = Number(writePose(sample, 65536+id, objects, i*8));
        });
        if (sample.state !== prior) {
          match = matchView(sample.state, content);
          update(match);
          prior = sample.state;
        }
        if (sample.status !== previousStatus) { status(sample.status); previousStatus = sample.status; }
      }
      if (match) scene.renderSkyline(bodies, objects, match, (now-priorTime)/1000, feed.own === null ? undefined : bodies[feed.own]);
      priorTime = now;
      animation = requestAnimationFrame(render);
    }
    animation = requestAnimationFrame(render);
  } catch (error) { unbind?.(); scene.dispose(); throw error; }
  return () => { cancelAnimationFrame(animation); unbind?.(); scene.dispose(); };
}
