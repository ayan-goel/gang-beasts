import { useId, type ReactNode, type RefObject } from 'react';
import './scoreboard.css';

interface Props {
  viewport: RefObject<HTMLElement | null>;
  className: string;
  children: ReactNode;
}

export function Scoreboard({viewport, className, children}: Props) {
  const hint = useId();
  return <section className={'scoreboard ' + className} aria-label="Scoreboard" aria-describedby={hint}
    tabIndex={0} onKeyDown={event => {
      if (event.key !== 'Escape' || event.repeat || event.altKey || event.ctrlKey || event.metaKey) return;
      const arena = viewport.current?.querySelector<HTMLCanvasElement>('canvas:not([inert])');
      if (arena) {
        event.preventDefault();
        arena.focus();
      }
    }}>
    {children}
    <small className="scoreboard-hint" id={hint}>Esc to return to the arena</small>
  </section>;
}
