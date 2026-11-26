export interface VolumeSettings { muted: boolean; master: number; effects: number; music: number }
export type VolumeChannel = Exclude<keyof VolumeSettings, 'muted'>;

const defaults: VolumeSettings = {muted:false, master:100, effects:100, music:60};
const key = 'skyline.audio.v2';

export function readVolume(storage?: Pick<Storage,'getItem'>): VolumeSettings {
  const result = {...defaults};
  try {
    storage ??= window.localStorage;
    result.muted = storage.getItem('skyline.audio.v1.muted') === 'true';
    const data = JSON.parse(storage.getItem(key) ?? 'null');
    if (typeof data?.muted === 'boolean') result.muted = data.muted;
    for (const channel of ['master','effects','music'] as const) {
      const value = data?.[channel];
      if (Number.isInteger(value) && value >= 0 && value <= 100) result[channel] = value;
    }
  } catch { /* A blocked store or damaged preference leaves the available defaults in effect. */ }
  return result;
}

export function saveVolume(value: VolumeSettings, storage?: Pick<Storage,'setItem'>) {
  try { (storage ?? window.localStorage).setItem(key, JSON.stringify(value)); }
  catch { /* The active tab keeps its mix if persistent storage is unavailable. */ }
}
