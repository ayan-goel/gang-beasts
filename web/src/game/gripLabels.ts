import type { SkylineMap } from '../rendering/SkylineView';

const arenaNames = {deck:'roof edge', panel:'walkway edge', crate:'crate', vent:'vent'};

export function arenaGripName(map: SkylineMap | undefined, id: number | undefined) {
  if (id === undefined) return undefined;
  const object = map?.objects.find(o => o.id === id);
  return object ? arenaNames[object.kind] : 'arena object';
}

export function fighterGripName(fighter: string, part: string) {
  return fighter + '’s ' + part.replaceAll('_',' ');
}

export function handGripLabels(target: (side: 0 | 1) => string | undefined) {
  return ([0,1] as const).flatMap(side => {
    const name = target(side);
    return name ? [(side === 0 ? 'Left' : 'Right') + ' hand on ' + name] : [];
  });
}
