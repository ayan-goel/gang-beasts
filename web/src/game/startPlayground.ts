import { LocalSimulation } from '../generated/wasm/brawler_wasm';
import configuration from '../generated/physics.json?raw';
import { FixtureScene } from '../rendering/FixtureScene';
import { Playground, type PlaygroundSnapshot } from './Playground';
import { RagdollSimulation } from './RagdollSimulation';
import type { SimulationPort } from './SimulationPort';

export type Scenario = 'cube' | 'ragdoll' | 'standing';

export function startPlayground(container: HTMLElement, publish: (state: PlaygroundSnapshot) => void, scenario: Scenario = 'cube') {
  const simulation: SimulationPort = scenario === 'cube' ? new LocalSimulation(configuration) : new RagdollSimulation(configuration, scenario === 'standing');
  let scene: FixtureScene | undefined;
  let frame = 0;
  let disposed = false;
  const dispose = () => {
    if (disposed) return;
    disposed = true;
    cancelAnimationFrame(frame);
    try { scene?.dispose(); } finally { simulation.free(); }
  };
  try {
    scene = new FixtureScene(container, simulation.rig?.definition, scenario === 'standing' ? 0.85 : 1.7);
    const liveScene = scene;
    const playground = new Playground(simulation);
    if (simulation.rig) playground.setPaused(true);
    let previousDisplay = 0;
    const render = () => liveScene.render(
      simulation.rig?.bodyStates() ?? simulation.fixture_state(),
      playground.showColliders ? simulation.collider_lines() : undefined,
      playground.showJoints ? simulation.rig?.jointStates() : undefined, simulation.rig?.selectedBody,
    );
    const refresh = () => {
      render();
      publish(playground.snapshot());
    };
    const animate = (now: number) => {
      playground.frame(now);
      render();
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
      dispose,
    };
  } catch (error) {
    // Startup may fail before React receives the disposal handle, including
    // during the first native-buffer render. Roll back every acquired resource.
    dispose();
    throw error;
  }
}

export type PlaygroundRuntime = ReturnType<typeof startPlayground>;
