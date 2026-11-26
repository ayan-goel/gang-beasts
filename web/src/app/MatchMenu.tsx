import { useId, useLayoutEffect, useRef, type KeyboardEvent, type ReactNode } from 'react';
import { createPortal } from 'react-dom';
import { MuteButton, VolumeControls } from '../audio/SkylineSound';
import { GraphicsSelect } from './GraphicsSelect';
import './matchMenu.css';

export function openMenuFromArena(event: KeyboardEvent, open: () => void) {
  if (!(event.target instanceof HTMLCanvasElement) || event.key !== 'Escape' ||
      event.repeat || event.altKey || event.metaKey || event.ctrlKey) return;
  event.preventDefault();
  open();
}

export function MatchMenu({local, children, resume}: {local?: boolean; children?: ReactNode; resume: () => void}) {
  const dialog = useRef<HTMLDialogElement>(null);
  const title = useId();
  useLayoutEffect(() => {
    const element = dialog.current!;
    element.showModal();
    return () => element.close();
  }, []);
  const dismiss = () => {
    dialog.current?.close();
    resume();
  };
  // Keep menu keystrokes outside the online arena's native spectator camera listener.
  return createPortal(<dialog className="match-menu-dialog" aria-labelledby={title} ref={dialog}
    onCancel={event => { event.preventDefault(); dismiss(); }}
    onKeyDown={event => { if (event.key === 'Escape' && event.repeat) event.preventDefault(); }}>
    <h2 id={title}>{local ? 'Game paused' : 'Match menu'}</h2>
    {!local && <p>The round continues while this menu is open.</p>}
    <button className="match-menu-return" onClick={dismiss}>{local ? 'Resume match' : 'Return to match'}</button>
    <div className="match-menu-settings"><GraphicsSelect /><MuteButton /></div>
    <VolumeControls />
    {children}
  </dialog>, document.body);
}
