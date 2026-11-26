import { useEffect, useRef, useState, type RefObject } from 'react';
import { prepareLedgePractice, type LedgePractice } from '../game/LedgePractice';
import { startDuel } from '../game/startDuel';
import { loadFighterModel } from '../rendering/fighter/loadFighterModel';

const feedback = {
  ready: 'Watch the walkway',
  falling: 'Below the roof edge',
  landing: 'Getting back on your feet',
  hanging: 'Holding the roof edge',
  recovered: 'Back on the roof',
  missed: 'Fell from the walkway',
};

export function LedgePracticePanel({ viewport }: { viewport: RefObject<HTMLDivElement | null> }) {
  const runtime = useRef<ReturnType<typeof startDuel> | null>(null);
  const [attempt, setAttempt] = useState(0);
  const [status, setStatus] = useState<LedgePractice['status']>();
  const [started, setStarted] = useState(false);
  const [error, setError] = useState('');
  useEffect(() => {
    const controller = new AbortController();
    async function prepare() {
      const model = await loadFighterModel();
      const practice = await prepareLedgePractice(controller.signal);
      if (controller.signal.aborted) { practice.duel.free(); return; }
      runtime.current = startDuel(viewport.current!, () => setStatus(practice.status), model, practice);
    }
    void prepare().catch(reason => { if (!controller.signal.aborted) setError(String(reason)); });
    return () => { controller.abort(); runtime.current?.dispose(); runtime.current = null; };
  }, [viewport, attempt]);

  const retry = () => { setStatus(undefined); setError(''); setStarted(false); setAttempt(n => n + 1); };
  return <section className="inspector-section" aria-label="Ledge rescue practice">
    <h3>Rescue Coral</h3>
    <p className="hint">Hold J and K as the walkway drops. Once you catch the roof, keep your hands held and hold Space to pull yourself up. Release both hands to let go.</p>
    {error ? <p role="alert">{error}</p> :
      <p role="status" aria-label="Rescue status">{status ? feedback[status] : 'Preparing the walkway…'}</p>}
    <div className="button-row">
      {!started && !error && <button className="primary" disabled={!status} onClick={() => {
        setStarted(true); runtime.current?.toggle();
      }}>Start rescue</button>}
      {(started || error) && <button onClick={retry}>Retry rescue</button>}
    </div>
  </section>;
}
