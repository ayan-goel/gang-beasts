import '../settings/applyTextSize';
import { assetWaitMs } from '../game/assetLoading';
import { loadPhysics } from '../game/loadPhysics';
import { physicsUnsupported, supportsPhysics } from '../game/physicsSupport';
import { StrictMode, type ReactElement } from 'react';
import { createRoot } from 'react-dom/client';
import '../style.css';

export function mountSimulation(view: ReactElement) {
  const root = createRoot(document.querySelector<HTMLElement>('#app')!);
  let pending: AbortController | undefined;
  const offerRetry = (message: string) => root.render(<div className="simulation-startup">
    <p role="alert">{message}</p>
    <button onClick={load}>Retry physics download</button><a href="/">Main menu</a>
  </div>);
  async function load() {
    if (!supportsPhysics()) {
      root.render(<div className="simulation-startup"><p role="alert">{physicsUnsupported}</p><a href="/">Main menu</a></div>);
      return;
    }
    pending?.abort();
    const attempt = pending = new AbortController();
    const timer = window.setTimeout(() => {
      if (!attempt.signal.aborted) offerRetry('Physics is taking longer than usual to load. You can keep waiting or retry.');
    }, assetWaitMs);
    root.render(<p role="status">Loading physics…</p>);
    try {
      await loadPhysics(attempt.signal);
      if (!attempt.signal.aborted) root.render(<StrictMode>{view}</StrictMode>);
    } catch {
      if (!attempt.signal.aborted) offerRetry('Could not load the physics engine. Check your connection and try again.');
    } finally { clearTimeout(timer); }
  }
  void load();
  return () => { pending?.abort(); root.unmount(); };
}
