import { useEffect, useRef } from 'react';
import { downloadJson } from '../game/downloadJson';
import type { InspectorProps } from './types';

export function Recordings({ state, act, reportError, sessionRevision }: InspectorProps & {
  reportError: (error: string) => void;
  sessionRevision: () => string;
}) {
  const request = useRef(0);
  useEffect(() => () => { request.current++; }, []);
  return <section className="inspector-section" aria-label="Recording and replay">
    <h2>Reproduce a run</h2>
    <p className="hint" id="replay-status">{state.replayEnd !== undefined
      ? `Replay ${state.replayComplete ? 'complete' : 'loaded'} · ${state.replayEnd} ticks. Reset to return to live play.`
      : state.recordingAvailable ? 'Recording completed ticks since the last reset.' : 'Recording limit reached. Reset to start a new log.'}</p>
    <button disabled={!state.recordingAvailable} onClick={() => act(p => downloadJson('fixture-recording-v1.json', p.exportRecording()))}>
      Export recording
    </button>
    <label className="field">Import recording
      <input type="file" accept=".json,application/json" onChange={async event => {
        const file = event.currentTarget.files?.[0];
        event.currentTarget.value = '';
        const current = ++request.current;
        const revision = sessionRevision();
        if (!file) return;
        if (file.size > 262_144) { reportError('Recording exceeds 256 KiB.'); return; }
        try {
          const json = await file.text();
          // Reset/tuning, a newer selection, or unmount supersedes this read.
          // Read the live revision, not React's potentially older render snapshot.
          if (current === request.current && revision === sessionRevision()) act(p => p.importRecording(json));
        } catch (error) {
          if (current === request.current && revision === sessionRevision()) reportError(String(error));
        }
      }} />
    </label>
    <p className="hint">Same engine build required. Up to 10 minutes and 1,024 impulses. Import starts paused at tick zero.</p>
  </section>;
}
