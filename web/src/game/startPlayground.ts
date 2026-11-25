import { LocalSimulation } from '../generated/wasm/brawler_wasm';
import configuration from '../generated/physics.json?raw';
import { FixtureScene } from '../rendering/FixtureScene';
import { Playground, type PlaygroundSnapshot } from './Playground';

export function startPlayground(container: HTMLElement, publish: (state: PlaygroundSnapshot) => void) {
  const simulation = new LocalSimulation(configuration);
  let scene: FixtureScene;
  try {
    scene = new FixtureScene(container);
  } catch (error) {
    simulation.free();
    throw error;
  }
  const playground = new Playground(simulation);
  let previousDisplay = 0;
  let frame = 0;
  const refresh = () => {
    scene.render(simulation.fixture_state(), playground.showColliders ? simulation.collider_lines() : undefined);
    publish(playground.snapshot());
  };
  const animate = (now: number) => {
    playground.frame(now);
    scene.render(simulation.fixture_state(), playground.showColliders ? simulation.collider_lines() : undefined);
    if (now - previousDisplay >= 250) {
      publish(playground.snapshot());
      previousDisplay = now;
    }
    frame = requestAnimationFrame(animate);
  };
  refresh();
  frame = requestAnimationFrame(animate);
  return {
    playground,
    act(action: (playground: Playground) => void) { action(playground); refresh(); },
    dispose() { cancelAnimationFrame(frame); scene.dispose(); simulation.free(); },
  };
}

export type PlaygroundRuntime = ReturnType<typeof startPlayground>;
