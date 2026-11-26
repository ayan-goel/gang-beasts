import { storedChoice } from '../settings/storedChoice';

export const graphicsPresets = ['auto', 'low', 'medium', 'high'] as const;
export type GraphicsPreset = typeof graphicsPresets[number];
export const {value: graphicsPreference, subscribe: subscribeGraphics, choose: chooseGraphics} =
  storedChoice('skyline.graphics.v1.preset', graphicsPresets, 'auto');
