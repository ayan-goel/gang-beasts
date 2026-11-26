import { useState } from 'react';
import { approachTicks, inspectStairFrame, maximumSettlingTicks, validStairFrame, walkingTicks } from '../game/stairInspection';
import type { InspectorProps } from './types';

export function StairInspection({ act, state }: InspectorProps) {
  const [settle, setSettle] = useState('60');
  const [elapsed, setElapsed] = useState('150');
  const enabled = state.rig?.controller?.enabled && settle !== '' && elapsed !== ''
    && validStairFrame(Number(settle), Number(elapsed));

  return <section aria-label="Stair inspection">
    <h2>Inspect a stair approach</h2>
    <p className="hint">Replay from spawn through the chosen frame. The fighter walks for {walkingTicks} ticks, then releases. Playback pauses at the frame you select.</p>
    <label className="field">Settling ticks
      <input type="number" min={0} max={maximumSettlingTicks} value={settle} onChange={event => setSettle(event.target.value)} />
    </label>
    <label className="field">Ticks after movement starts
      <input type="number" min={0} max={approachTicks} value={elapsed} onChange={event => setElapsed(event.target.value)} />
    </label>
    <button disabled={!enabled} onClick={() => act(p => inspectStairFrame(p, Number(settle), Number(elapsed)))}>Inspect frame</button>
  </section>;
}
