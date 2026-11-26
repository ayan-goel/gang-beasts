import { useState } from 'react';
import { defaultWalkTicks, maximumWalkTicks, stopTicks, inspectStairFrame, maximumSettlingTicks, validStairFrame } from '../game/stairInspection';
import type { InspectorProps } from './types';

export function StairInspection({ act, state }: InspectorProps) {
  const [settle, setSettle] = useState('60');
  const [walk, setWalk] = useState(String(defaultWalkTicks));
  const [elapsed, setElapsed] = useState('150');
  const enabled = state.rig?.controller?.enabled && settle !== '' && walk !== '' && elapsed !== ''
    && validStairFrame(Number(settle), Number(elapsed), Number(walk));

  return <section aria-label="Stair inspection">
    <h2>Inspect a stair approach</h2>
    <p className="hint">Choose when the fighter releases forward and which frame to inspect. Each second of play is 60 ticks.</p>
    <label className="field">Settling ticks
      <input type="number" min={0} max={maximumSettlingTicks} value={settle} onChange={event => setSettle(event.target.value)} />
    </label>
    <label className="field">Ticks holding forward
      <input type="number" min={0} max={maximumWalkTicks} value={walk} onChange={event => setWalk(event.target.value)} />
    </label>
    <label className="field">Ticks after movement starts
      <input type="number" min={0} max={Number(walk) + stopTicks} value={elapsed} onChange={event => setElapsed(event.target.value)} />
    </label>
    <button disabled={!enabled} onClick={() => act(p => inspectStairFrame(p, Number(settle), Number(elapsed), Number(walk)))}>Inspect frame</button>
  </section>;
}
