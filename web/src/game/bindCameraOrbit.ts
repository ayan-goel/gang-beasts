import type { GroupCamera } from '../rendering/GroupCamera';

export function bindCameraOrbit(canvas: HTMLCanvasElement, camera: GroupCamera, allowed: () => boolean) {
  canvas.tabIndex = 0;
  canvas.setAttribute('aria-keyshortcuts', 'Z X');
  const keydown = (event: KeyboardEvent) => {
    if (event.repeat || event.ctrlKey || event.altKey || event.metaKey || !allowed()) return;
    if (event.code !== 'KeyZ' && event.code !== 'KeyX') return;
    camera.turn(event.code === 'KeyX' ? 1 : -1);
    event.preventDefault();
  };
  canvas.addEventListener('keydown', keydown);
  return () => canvas.removeEventListener('keydown', keydown);
}
