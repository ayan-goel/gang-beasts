import type { SkylineContent } from '../lobby/content';
import { FixtureScene } from '../rendering/FixtureScene';
import { GraphicsUnavailable } from '../rendering/createSceneRenderer';

export class PreparedArena {
  private scene?: FixtureScene;

  async prepare(content: SkylineContent, signal: AbortSignal) {
    const staging = document.createElement('div');
    let scene: FixtureScene;
    try {
      scene = new FixtureScene(staging, content.fighter, {online:true, map:content.map, model:content.model});
    } catch (failure) {
      // Allow room entry so a player can recover graphics without losing their seat.
      if (failure instanceof GraphicsUnavailable) return;
      throw failure;
    }
    try {
      await scene.prepareSkyline(signal);
      signal.throwIfAborted();
      this.scene = scene;
    } catch (failure) {
      scene.dispose();
      throw failure;
    }
  }

  take() {
    const scene = this.scene;
    this.scene = undefined;
    if (scene?.graphicsInterrupted) {
      scene.dispose();
      return;
    }
    return scene;
  }

  dispose() {
    this.scene?.dispose();
    this.scene = undefined;
  }
}
