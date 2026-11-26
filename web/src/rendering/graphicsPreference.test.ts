import { afterEach, expect, it, vi } from 'vitest';

afterEach(() => { vi.unstubAllGlobals(); vi.resetModules(); });

it('uses automatic quality for an unknown saved value and follows storage updates until disposal', async () => {
  const storage = new Map<string,string>([['skyline.graphics.v1.preset', 'ultra']]);
  const key = 'skyline.graphics.v1.preset';
  const browser = new EventTarget();
  vi.stubGlobal('window', browser);
  vi.stubGlobal('localStorage', {getItem:(name: string) => storage.get(name), setItem:(name: string, value: string) => storage.set(name, value)});
  const {graphicsPreference, subscribeGraphics, chooseGraphics} = await import('./graphicsPreference');
  expect(graphicsPreference()).toBe('auto');
  const notify = vi.fn();
  const stop = subscribeGraphics(notify);
  chooseGraphics('low');
  chooseGraphics('low');
  expect(notify).toHaveBeenCalledTimes(1);
  expect(storage.get(key)).toBe('low');
  storage.set(key, 'high');
  browser.dispatchEvent(Object.assign(new Event('storage'), {key}));
  expect(graphicsPreference()).toBe('high');
  expect(notify).toHaveBeenCalledTimes(2);
  stop();
  storage.clear();
  browser.dispatchEvent(Object.assign(new Event('storage'), {key:null}));
  expect(notify).toHaveBeenCalledTimes(2);
  const resumed = subscribeGraphics(notify);
  expect(graphicsPreference()).toBe('auto');
  resumed();
});

it('keeps a quality choice usable on a page that denies storage', async () => {
  const denied = () => { throw new Error('Storage denied'); };
  vi.stubGlobal('localStorage', {getItem:denied, setItem:denied});
  const {graphicsPreference, chooseGraphics} = await import('./graphicsPreference');
  expect(graphicsPreference()).toBe('auto');
  chooseGraphics('medium');
  expect(graphicsPreference()).toBe('medium');
});
