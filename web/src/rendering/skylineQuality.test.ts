import { expect, it } from 'vitest';
import { skylineQuality } from './skylineQuality';

it('selects the low render scale and basic shadows for software WebGL', () => {
  for (const device of ['ANGLE (Google SwiftShader Device)', 'Mesa llvmpipe (LLVM)', 'Microsoft Basic Render Driver']) {
    expect(skylineQuality(device, 2)).toEqual({pixelRatio:.75, shadowSize:512, basic:true});
    expect(skylineQuality(device, .8).pixelRatio).toBeCloseTo(.6);
  }
});

it('preserves medium rendering on hardware and when the driver name is unavailable', () => {
  for (const device of ['Apple M3 Pro', 'ANGLE (Intel Iris Xe)', '']) {
    expect(skylineQuality(device, 2)).toEqual({pixelRatio:1.5, shadowSize:1024, basic:false});
    expect(skylineQuality(device, 1).pixelRatio).toBe(1);
  }
});

it('applies explicit choices before automatic driver selection and scales once', () => {
  expect(skylineQuality('Apple GPU', 2, 'low')).toEqual({pixelRatio:.75, shadowSize:512, basic:true});
  expect(skylineQuality('SwiftShader', 3, 'medium')).toEqual({pixelRatio:1.5, shadowSize:1024, basic:false});
  expect(skylineQuality('SwiftShader', 3, 'high')).toEqual({pixelRatio:2, shadowSize:2048, basic:false});
  expect(skylineQuality('Apple GPU', .8, 'high').pixelRatio).toBe(.8);
});
