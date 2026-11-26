import { expect, it } from 'vitest';
import { readVolume, saveVolume } from './volumeSettings';

function store(initial: Record<string,string> = {}) {
  return {
    getItem: (key: string) => initial[key] ?? null,
    setItem: (key: string, value: string) => { initial[key] = value; },
  };
}

it('keeps an existing mute choice until explicitly changed and round-trips separate channel levels', () => {
  const storage = store({'skyline.audio.v1.muted':'true'});
  const initial = readVolume(storage);
  expect(initial).toEqual({muted:true,master:100,effects:100,music:60});
  saveVolume({muted:false,master:73,effects:17,music:0}, storage);
  expect(readVolume(storage)).toEqual({muted:false,master:73,effects:17,music:0});
});

it('repairs individual invalid levels and falls back safely on unreadable preferences', () => {
  const storage = store({'skyline.audio.v1.muted':'true', 'skyline.audio.v2':JSON.stringify({music:-1,master:'loud',effects:25})});
  expect(readVolume(storage)).toEqual({muted:true,master:100,effects:25,music:60});
  storage.setItem('skyline.audio.v2', '{');
  expect(readVolume(storage).muted).toBe(true);
  expect(readVolume({getItem() { throw Error(); }})).toEqual({muted:false,master:100,effects:100,music:60});
});
