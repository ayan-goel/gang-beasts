import { FixtureScene } from '../rendering/FixtureScene';
import type { MatchView } from '../rendering/SkylineView';
import type { SkylineContent } from '../lobby/content';
import type { Snapshot } from '../net/snapshot';
import type { MatchFeed } from './MatchFeed';

const colors = ['#f38e66','#64b7dd','#e8bd59','#7dd3b1','#bd9ae6','#e69eb9','#b5d87e','#8cacc1'];
const phases = ['countdown','playing','round_results','match_results'] as const;

export function matchView(frame: Snapshot, content: SkylineContent): MatchView {
  const t = frame.arenaTick;
  const s = content.map.schedule;
  return {phase: phases[frame.phase]!, round: frame.number, epoch: String(frame.round), remaining: frame.remaining,
    scores: [frame.fighters[0]!.score, frame.fighters[1]!.score], winner: frame.winner,
    visible: [frame.fighters[0]!.consciousness !== 3, frame.fighters[1]!.consciousness !== 3],
    timeLeft: Math.max(0, (t < s.gust_tick ? s.gust_tick : s.end_tick) - t),
    hazard: t >= s.gust_tick ? 'gust' : t >= s.east_tick ? 'gust_warning' : t >= s.west_tick ? 'east_warning' : t >= s.warn_tick ? 'west_warning' : 'quiet'};
}

export function startRemoteScene(container: HTMLElement, content: SkylineContent, feed: MatchFeed, update: (match: MatchView) => void) {
  const scene = new FixtureScene(container, content.fighter, {map: content.map, color: colors[feed.fighters[0].color]});
  const bodies = [new Float32Array(15*14), new Float32Array(15*14)] as const;
  const objects = new Float32Array(8*content.map.objects.length);
  let animation = 0, priorTime = performance.now(), prior: Snapshot | null = null, match: MatchView | null = null;
  try {
    scene.addOpponent(content.fighter, colors[feed.fighters[1].color]);
    function render(now: number) {
      const frame = feed.latest;
      if (frame && frame !== prior) {
        for (const [slot, buffer] of bodies.entries()) {
          for (let i=0; i<15; i++) {
            const pose = frame.bodies.get(1+slot*16+i)!;
            buffer.set(pose.position, i*14);
            buffer.set(pose.rotation, i*14+3);
          }
        }
        content.map.objects.forEach(({id}, i) => {
          const pose = frame.bodies.get(65536+id);
          objects[i*8+7] = Number(!!pose);
          if (pose) { objects.set(pose.position,i*8); objects.set(pose.rotation,i*8+3); }
        });
        match = matchView(frame, content);
        update(match);
        prior = frame;
      }
      if (match) scene.renderSkyline(bodies[0],bodies[1],objects,match,(now-priorTime)/1000);
      priorTime = now;
      animation = requestAnimationFrame(render);
    }
    animation = requestAnimationFrame(render);
  } catch (error) { scene.dispose(); throw error; }
  return () => { cancelAnimationFrame(animation); scene.dispose(); };
}
