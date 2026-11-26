import { arenaGripName, fighterGripName, handGripLabels } from '../game/gripLabels';
import type { SkylineMap } from '../rendering/SkylineView';
import { groupOf } from './SnapshotTimeline';

export function onlineGripLabels(targets: readonly [number,number], names: readonly string[],
  bodies: readonly {part: string}[], map: SkylineMap) {
  return handGripLabels(side => {
    const id = targets[side];
    if (!id) return undefined;
    if (id > 65536) return arenaGripName(map, id-65536);
    const slot = groupOf(id);
    return fighterGripName(names[slot]!, bodies[(id-1)%16]!.part);
  });
}
