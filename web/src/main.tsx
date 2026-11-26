import { SkylineSound } from './audio/SkylineSound';
import { createRoot } from 'react-dom/client';
import { StrictMode } from 'react';
import init from './generated/wasm/brawler_wasm';
import { SkylineGame } from './app/SkylineGame';
import { App } from './app/App';
import './style.css';

const play = ['/play', '/play/', '/play/index.html'].includes(location.pathname);
if (play) document.title = 'Skyline Melee · Local multiplayer';
const root = createRoot(document.querySelector<HTMLElement>('#app')!);
let disposed = false;
void init().then(() => {
  if (!disposed) root.render(<StrictMode>{play ? <SkylineSound><SkylineGame /></SkylineSound> : <App />}</StrictMode>);
}).catch(error => {
  if (!disposed) root.render(<p role="alert">Unable to load WebAssembly. {String(error)}</p>);
});
if (import.meta.hot) import.meta.hot.dispose(() => { disposed = true; root.unmount(); });
