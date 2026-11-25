import type { MovementInput } from './MovementInput';

export function bindMovementInput(canvas: HTMLCanvasElement, input: MovementInput) {
  canvas.tabIndex = 0;
  canvas.setAttribute('aria-description', 'Use WASD or arrow keys to move, Shift to run, Space to jump, and J or K for the left or right hand.');
  const keydown = (event: KeyboardEvent) => {
    if (event.ctrlKey || event.metaKey || event.altKey) return;
    if (input.key(event.code, true, event.repeat)) event.preventDefault();
  };
  const keyup = (event: KeyboardEvent) => { if (input.key(event.code, false)) event.preventDefault(); };
  const mouse = (event: MouseEvent) => {
    if (event.button !== 0 && event.button !== 2) return;
    const down = event.type === 'mousedown';
    if (input.hands.hold(event.button === 0 ? 0 : 1, 'mouse', down) && down) {
      canvas.focus({preventScroll:true});
      event.preventDefault();
    }
  };
  const context = (event: MouseEvent) => event.preventDefault();
  // Mouse down/up report each button in a chord. A pointerdown alone reports only the first.
  canvas.addEventListener('mousedown', mouse);
  window.addEventListener('mouseup', mouse);
  canvas.addEventListener('contextmenu', context);
  const clear = () => input.clear();
  const visibility = () => { if (document.hidden) clear(); };
  canvas.addEventListener('keydown', keydown); canvas.addEventListener('keyup', keyup);
  canvas.addEventListener('blur', clear); window.addEventListener('blur', clear);
  document.addEventListener('visibilitychange', visibility);
  return () => {
    canvas.removeEventListener('mousedown', mouse);
    window.removeEventListener('mouseup', mouse);
    canvas.removeEventListener('contextmenu', context);
    canvas.removeEventListener('keydown', keydown); canvas.removeEventListener('keyup', keyup);
    canvas.removeEventListener('blur', clear); window.removeEventListener('blur', clear);
    document.removeEventListener('visibilitychange', visibility);
    clear();
  };
}
