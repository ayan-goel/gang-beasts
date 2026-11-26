import { nextSurvivor } from './spectatorCamera';

interface Props {
  names: string[];
  visible: boolean[];
  target: number | null;
  choose: (slot: number | null) => void;
}

export function SpectatorControls({names, visible, target, choose}: Props) {
  const survivors = visible.filter(Boolean).length;
  return <div className="spectator-camera" role="group" aria-label="Spectator camera">
    <output aria-label="Camera view">{target === null ? (survivors ? 'Group view' : 'Arena overview') : 'Following ' + names[target]}</output>
    <button disabled={!survivors || (survivors === 1 && target !== null)} onClick={() => choose(nextSurvivor(visible, target, -1))}>Previous fighter</button>
    <button disabled={!survivors || (survivors === 1 && target !== null)} onClick={() => choose(nextSurvivor(visible, target, 1))}>Next fighter</button>
    <button disabled={target === null} onClick={() => choose(null)}>Group view</button>
  </div>;
}
