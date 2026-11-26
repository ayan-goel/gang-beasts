export const graphicsPresets = ['auto', 'low', 'medium', 'high'] as const;
export type GraphicsPreset = typeof graphicsPresets[number];
const key = 'skyline.graphics.v1.preset';
const listeners = new Set<() => void>();
let current: GraphicsPreset | undefined;
let unsaved = false;

function storedPreset(): GraphicsPreset {
  try {
    const stored = localStorage.getItem(key);
    return graphicsPresets.find(preset => preset === stored) ?? 'auto';
  } catch { return current ?? 'auto'; }
}

export function graphicsPreference() { return current ??= storedPreset(); }

function publish(preset: GraphicsPreset) {
  if (current === preset) return;
  current = preset;
  for (const listener of listeners) listener();
}

function onStorage(event: StorageEvent) {
  if (event.key === key || event.key === null) {
    unsaved = false;
    publish(storedPreset());
  }
}

export function subscribeGraphics(listener: () => void) {
  const first = !listeners.size;
  listeners.add(listener);
  if (first) {
    window.addEventListener('storage', onStorage);
    if (!unsaved) publish(storedPreset());
  }
  return () => {
    listeners.delete(listener);
    if (!listeners.size) window.removeEventListener('storage', onStorage);
  };
}

export function chooseGraphics(preset: GraphicsPreset) {
  try { localStorage.setItem(key, preset); unsaved = false; } catch { unsaved = true; }
  publish(preset);
}
