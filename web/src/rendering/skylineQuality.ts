import type { GraphicsPreset } from './graphicsPreference';

export function skylineQuality(driver: string, deviceRatio: number, preset: GraphicsPreset = 'auto') {
  const software = /swiftshader|llvmpipe|softpipe|software rasterizer|microsoft basic render driver/i.test(driver);
  const quality = preset === 'auto' ? (software ? 'low' : 'medium') : preset;
  if (quality === 'low') return {pixelRatio:.75 * Math.min(deviceRatio, 1), shadowSize:512, basic:true};
  const high = quality === 'high';
  return {pixelRatio:Math.min(deviceRatio, high ? 2 : 1.5), shadowSize:high ? 2048 : 1024, basic:false};
}

export function graphicsDriver(gl: WebGL2RenderingContext | WebGLRenderingContext) {
  const info = gl.getExtension('WEBGL_debug_renderer_info');
  return info ? String(gl.getParameter(info.UNMASKED_RENDERER_WEBGL)) : '';
}
