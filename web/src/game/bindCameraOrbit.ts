import type { GroupCamera } from '../rendering/GroupCamera';

export function bindCameraOrbit(canvas: HTMLCanvasElement, camera: GroupCamera, allowed: () => boolean, scope: HTMLElement = canvas) {
  canvas.tabIndex = 0;
  canvas.setAttribute('aria-keyshortcuts', 'Z X');
  const keydown = (event: KeyboardEvent) => {
    if (event.repeat || event.ctrlKey || event.altKey || event.metaKey || !allowed()) return;
    if (event.code !== 'KeyZ' && event.code !== 'KeyX') return;
    camera.turn(event.code === 'KeyX' ? 1 : -1);
    event.preventDefault();
  };
  scope.addEventListener('keydown', keydown);
  return () => scope.removeEventListener('keydown', keydown);
}
