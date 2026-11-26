import { storedChoice } from '../settings/storedChoice';

export const motionChoices = ['system', 'reduced', 'full'] as const;
export const cameraMotion = storedChoice('skyline.camera.v1.motion', motionChoices, 'system');

export function observeCameraMotion(apply: (reduced: boolean) => void) {
  const media = window.matchMedia('(prefers-reduced-motion: reduce)');
  const refresh = () => {
    const choice = cameraMotion.value();
    apply(choice === 'system' ? media.matches : choice === 'reduced');
  };
  const unsubscribe = cameraMotion.subscribe(refresh);
  media.addEventListener('change', refresh);
  refresh();
  return () => { media.removeEventListener('change', refresh); unsubscribe(); };
}
