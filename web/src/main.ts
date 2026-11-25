import init, { LocalSimulation } from './generated/wasm/brawler_wasm';
import configuration from './generated/physics.json?raw';
import { FixedClock } from './game/FixedClock';
import { FixtureScene } from './rendering/FixtureScene';
import './style.css';

const app = document.querySelector<HTMLElement>('#app')!;
let dispose = () => {};

async function boot(): Promise<void> {
  await init();
  const simulation = new LocalSimulation(configuration);
  app.innerHTML = `
    <header>
      <div><p class="eyebrow">Physics Brawler</p><h1>Development playground</h1></div>
      <span class="badge">FOUNDATION / 01</span>
    </header>
    <div class="workspace">
      <section class="viewport" aria-label="Physics scene">
        <div class="scene-label"><p class="eyebrow">Test environment</p>
        <h2>Floating platform</h2><p>One body. Real contact. Fixed timestep.</p></div>
      </section>
      <aside aria-label="Simulation diagnostics">
        <h2>Live simulation</h2>
        <p role="status">Simulation ready</p>
        <dl>
          <div><dt>Engine</dt><dd>Rust / Rapier</dd></div>
          <div><dt>Runtime</dt><dd>WebAssembly</dd></div>
          <div><dt>Fixed rate</dt><dd>60 Hz</dd></div>
          <div><dt>Completed tick</dt><dd id="tick">0</dd></div>
          <div><dt>Body height</dt><dd id="height">4.000 m</dd></div>
          <div><dt>Physics step</dt><dd id="timing">—</dd></div>
        </dl>
        <p class="note">Foundation fixture: a one-kilogram body falls onto a solid platform.
        Character controls and combat arrive in the next development slices.</p>
      </aside>
    </div>`;
  let scene: FixtureScene;
  try {
    scene = new FixtureScene(app.querySelector<HTMLElement>('.viewport')!);
  } catch (error) {
    simulation.free();
    throw error;
  }
  const tick = app.querySelector<HTMLElement>('#tick')!;
  const height = app.querySelector<HTMLElement>('#height')!;
  const timing = app.querySelector<HTMLElement>('#timing')!;
  const clock = new FixedClock();
  let previousTime = performance.now();
  let previousDisplay = 0;
  let frame = 0;
  let lastStepTime = 0;
  const render = (now: number) => {
    const steps = clock.advance((now - previousTime) / 1000);
    previousTime = now;
    const start = performance.now();
    for (let i = 0; i < steps; i++) simulation.step();
    if (steps) lastStepTime = (performance.now() - start) / steps;
    const position = simulation.position();
    scene.render(position);
    if (now - previousDisplay >= 250) {
      tick.textContent = simulation.tick().toString();
      height.textContent = `${position[1]!.toFixed(3)} m`;
      timing.textContent = `${lastStepTime.toFixed(2)} ms`;
      previousDisplay = now;
    }
    frame = requestAnimationFrame(render);
  };
  frame = requestAnimationFrame(render);
  dispose = () => {
    cancelAnimationFrame(frame);
    scene.dispose();
    simulation.free();
  };
}

void boot().catch(error => {
  app.replaceChildren();
  const message = document.createElement('p');
  message.setAttribute('role', 'alert');
  message.textContent = `Unable to start the playground. WebAssembly and WebGL 2 are required. ${String(error)}`;
  app.append(message);
});
if (import.meta.hot) import.meta.hot.dispose(() => dispose());
