import { physicsUnsupported, supportsPhysics } from '../game/physicsSupport';
import { StrictMode, type ReactElement } from 'react';
import { createRoot } from 'react-dom/client';
import init from '../generated/wasm/brawler_wasm';
import '../style.css';

export function mountSimulation(view: ReactElement) {
  const root = createRoot(document.querySelector<HTMLElement>('#app')!);
  let disposed = false;
  let loading = false;
  async function load() {
    if (loading || disposed) return;
    if (!supportsPhysics()) {
      root.render(<div className="simulation-startup"><p role="alert">{physicsUnsupported}</p><a href="/">Main menu</a></div>);
      return;
    }
    loading = true;
    root.render(<p role="status">Loading physics…</p>);
    try {
      await init();
      if (!disposed) root.render(<StrictMode>{view}</StrictMode>);
    } catch {
      if (!disposed) root.render(<div className="simulation-startup">
        <p role="alert">Could not load the physics engine. Check your connection and try again.</p>
        <button onClick={load}>Retry physics download</button><a href="/">Main menu</a>
      </div>);
    } finally { loading = false; }
  }
  void load();
  return () => { disposed = true; root.unmount(); };
}
