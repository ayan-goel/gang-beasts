import type { MovementInput } from './MovementInput';

export function bindMovementInput(canvas: HTMLCanvasElement, input: MovementInput) {
  canvas.tabIndex = 0;
  canvas.setAttribute('aria-description', 'Use WASD or arrow keys to move, Shift to run, and Space to jump.');
  const keydown = (event: KeyboardEvent) => {
    if (event.ctrlKey || event.metaKey || event.altKey) return;
    if (input.key(event.code, true, event.repeat)) event.preventDefault();
  };
  const keyup = (event: KeyboardEvent) => { if (input.key(event.code, false)) event.preventDefault(); };
  const clear = () => input.clear();
  const visibility = () => { if (document.hidden) clear(); };
  canvas.addEventListener('keydown', keydown); canvas.addEventListener('keyup', keyup);
  canvas.addEventListener('blur', clear); window.addEventListener('blur', clear);
  document.addEventListener('visibilitychange', visibility);
  return () => {
    canvas.removeEventListener('keydown', keydown); canvas.removeEventListener('keyup', keyup);
    canvas.removeEventListener('blur', clear); window.removeEventListener('blur', clear);
    document.removeEventListener('visibilitychange', visibility);
    clear();
  };
}
