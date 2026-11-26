import { afterEach, expect, it, vi } from 'vitest';

afterEach(() => { vi.unstubAllGlobals(); vi.resetModules(); });

it('follows the current device setting until overridden and unregisters both listeners', async () => {
  const browser = new EventTarget();
  const media = Object.assign(new EventTarget(), {matches:true});
  vi.stubGlobal('window', Object.assign(browser, {matchMedia:() => media}));
  const storage = new Map<string,string>();
  vi.stubGlobal('localStorage', {getItem:(key:string) => storage.get(key), setItem:(k:string,v:string) => storage.set(k,v)});
  const {cameraMotion, observeCameraMotion} = await import('./cameraMotion');
  const apply = vi.fn();
  const stop = observeCameraMotion(apply);
  expect(apply).toHaveBeenLastCalledWith(true);
  media.matches = false;
  media.dispatchEvent(new Event('change'));
  expect(apply).toHaveBeenLastCalledWith(false);
  cameraMotion.choose('reduced');
  expect(apply).toHaveBeenLastCalledWith(true);
  cameraMotion.choose('full');
  media.matches = true;
  media.dispatchEvent(new Event('change'));
  expect(apply).toHaveBeenLastCalledWith(false);
  storage.clear();
  browser.dispatchEvent(Object.assign(new Event('storage'), {key:null}));
  expect(apply).toHaveBeenLastCalledWith(true);
  stop();
  apply.mockClear();
  media.dispatchEvent(new Event('change'));
  cameraMotion.choose('full');
  expect(apply).not.toHaveBeenCalled();
});
