import { WebGLRenderer } from 'three';

export class GraphicsUnavailable extends Error {
  constructor(cause: unknown) {
    super('WebGL 2 could not start. Check that hardware acceleration is enabled or try a browser with WebGL 2 support.', {cause});
  }
}

export function createSceneRenderer() {
  try { return new WebGLRenderer({antialias:true}); }
  catch (cause) { throw new GraphicsUnavailable(cause); }
}
