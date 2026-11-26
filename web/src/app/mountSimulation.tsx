import { StrictMode, type ReactElement } from 'react';
import { createRoot } from 'react-dom/client';
import init from '../generated/wasm/brawler_wasm';
import '../style.css';

export function mountSimulation(view: ReactElement) {
  const root = createRoot(document.querySelector<HTMLElement>('#app')!);
  let disposed = false;
  void init().then(() => {
    if (!disposed) root.render(<StrictMode>{view}</StrictMode>);
  }).catch(error => {
    if (!disposed) root.render(<p role="alert">Unable to load WebAssembly. {String(error)}</p>);
  });
  return () => { disposed = true; root.unmount(); };
}
