// Installed before the production bundle so Three's existing devtools hook sees the match renderer.
export function installObserver({preset, warmup, seconds}) {
  localStorage.setItem('skyline.graphics.v1.preset', preset);
  const events = new EventTarget();
  window.__THREE_DEVTOOLS__ = events;
  const observation = {intervals:[], submissionIntervals:[], callbackTimes:[], drawTimes:[], calls:[], triangles:[], hidden:false};
  window.skylineProfile = observation;
  let start, previous, previousSubmission;
  const schedule = window.requestAnimationFrame.bind(window);
  window.requestAnimationFrame = callback => schedule(timestamp => {
    const samples = observation.drawTimes.length;
    const before = performance.now();
    callback(timestamp);
    if (samples !== observation.drawTimes.length) observation.callbackTimes.push(performance.now() - before);
  });
  window.startProfile = () => { start = performance.now() + warmup * 1000; };
  document.addEventListener('visibilitychange', () => { if (document.hidden) observation.hidden = true; });
  events.addEventListener('observe', ({detail:renderer}) => {
    if (!renderer.isWebGLRenderer) return;
    const original = renderer.render;
    renderer.render = function(...args) {
      const before = performance.now();
      const result = original.apply(this, args);
      const cost = performance.now() - before;
      const timestamp = document.timeline.currentTime;
      if (start !== undefined && timestamp >= start && timestamp < start + seconds * 1000) {
        if (previous !== undefined) {
          observation.intervals.push(timestamp - previous);
          observation.submissionIntervals.push(before - previousSubmission);
        }
        previous = timestamp;
        previousSubmission = before;
        observation.drawTimes.push(cost);
        observation.calls.push(renderer.info.render.calls);
        observation.triangles.push(renderer.info.render.triangles);
        if (!observation.gpu) {
          const gl = renderer.getContext();
          const ext = gl.getExtension('WEBGL_debug_renderer_info');
          observation.gpu = ext ? gl.getParameter(ext.UNMASKED_RENDERER_WEBGL) : 'unavailable';
          observation.buffer = [gl.drawingBufferWidth,gl.drawingBufferHeight];
          const light = args[0].children.find(object => object.isDirectionalLight);
          observation.shadow = light?.shadow.map?.width;
          const skeletons = new Set();
          args[0].traverseVisible(object => { if (object.isSkinnedMesh) skeletons.add(object.skeleton); });
          observation.skeletons = skeletons.size;
        }
      }
      return result;
    };
  });
}
