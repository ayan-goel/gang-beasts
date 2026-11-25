import { createRoot } from 'react-dom/client';
import { StrictMode } from 'react';
import init from './generated/wasm/brawler_wasm';
import { App } from './app/App';
import './style.css';

const root = createRoot(document.querySelector<HTMLElement>('#app')!);
let disposed = false;
void init().then(() => {
  if (!disposed) root.render(<StrictMode><App /></StrictMode>);
}).catch(error => {
  if (!disposed) root.render(<p role="alert">Unable to load WebAssembly. {String(error)}</p>);
});
if (import.meta.hot) import.meta.hot.dispose(() => { disposed = true; root.unmount(); });
