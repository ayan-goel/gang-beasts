export function skylineQuality(driver: string, deviceRatio: number) {
  const basic = /swiftshader|llvmpipe|softpipe|software rasterizer|microsoft basic render driver/i.test(driver);
  return basic
    ? {pixelRatio:.75 * Math.min(deviceRatio, 1), shadowSize:512, basic}
    : {pixelRatio:Math.min(deviceRatio, 1.5), shadowSize:1024, basic};
}

export function graphicsDriver(gl: WebGL2RenderingContext | WebGLRenderingContext) {
  const info = gl.getExtension('WEBGL_debug_renderer_info');
  return info ? String(gl.getParameter(info.UNMASKED_RENDERER_WEBGL)) : '';
}
