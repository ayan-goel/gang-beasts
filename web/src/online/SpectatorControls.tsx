import { nextSurvivor } from './spectatorCamera';
import { useLayoutEffect, useRef } from 'react';

interface Props {
  names: string[];
  visible: boolean[];
  target: number | null;
  choose: (slot: number | null) => void;
}

export function SpectatorControls({names, visible, target, choose}: Props) {
  const controls = useRef<HTMLDivElement>(null);
  const used = useRef(false);
  useLayoutEffect(() => {
    const group = controls.current!;
    return () => {
      // Removing focused camera controls at respawn would leave movement keys addressed to the page.
      if (group.contains(document.activeElement) || (used.current && document.activeElement === document.body)) {
        group.closest('section')?.querySelector('canvas')?.focus({preventScroll:true});
      }
    };
  }, []);
  const survivors = visible.filter(Boolean).length;
  return <div ref={controls} className="spectator-camera" role="group" aria-label="Spectator camera" onFocus={() => { used.current = true; }}>
    <output aria-label="Camera view">{target === null ? (survivors ? 'Group view' : 'Arena overview') : 'Following ' + names[target]}</output>
    <button disabled={!survivors || (survivors === 1 && target !== null)} onClick={() => choose(nextSurvivor(visible, target, -1))}>Previous fighter</button>
    <button disabled={!survivors || (survivors === 1 && target !== null)} onClick={() => choose(nextSurvivor(visible, target, 1))}>Next fighter</button>
    <button disabled={target === null} onClick={() => choose(null)}>Group view</button>
  </div>;
}
