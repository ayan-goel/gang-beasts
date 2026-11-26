import type { Page } from '@playwright/test';

export async function observeSkinDraws(page: Page, count: number) {
  await page.addInitScript(target => {
    const textures = new Set<WebGLTexture>();
    Object.defineProperty(window, 'drawnFighterSkeletons', {get: () => textures.size});
    const gl = WebGL2RenderingContext.prototype;
    const {getUniformLocation, uniform1i, useProgram, activeTexture, bindTexture, drawElements} = gl;
    const samplers = new WeakMap<WebGLUniformLocation, WebGLProgram>();
    const units = new WeakMap<WebGLProgram, number>();
    const bound = new Map<number, WebGLTexture | null>();
    let current: WebGLProgram | null = null, active = 0;
    // Observe existing bindings. Synchronous GPU queries distort startup on software renderers.
    gl.getUniformLocation = function(program, name) {
      const location = getUniformLocation.call(this, program, name);
      if (name === 'boneTexture' && location) samplers.set(location, program);
      return location;
    };
    gl.uniform1i = function(location, unit) {
      uniform1i.call(this, location, unit);
      const program = location && samplers.get(location);
      if (program) units.set(program, unit);
    };
    gl.useProgram = function(program) { useProgram.call(this, program); current = program; };
    gl.activeTexture = function(unit) { activeTexture.call(this, unit); active = unit - this.TEXTURE0; };
    gl.bindTexture = function(target, texture) {
      bindTexture.call(this, target, texture);
      if (target === this.TEXTURE_2D) bound.set(active, texture);
    };
    gl.drawElements = function(...args) {
      drawElements.apply(this, args);
      if (!current || textures.size >= target) return;
      const sampler = units.get(current);
      const texture = sampler === undefined ? null : bound.get(sampler);
      if (texture) textures.add(texture);
    };
  }, count);
  return () => page.evaluate(() => Number(Reflect.get(window, 'drawnFighterSkeletons') ?? 0));
}
