import { expect, test } from '@playwright/test';

test('a moving crate reveals only the covered parts of a live fighter', async ({page}) => {
  await page.goto('/rooms/');
  const pixels = await page.evaluate(async () => {
    const {FixtureScene} = await import(/* @vite-ignore */ String('/src/rendering/FixtureScene.ts'));
    const {loadFighterModel} = await import(/* @vite-ignore */ String('/src/rendering/fighter/loadFighterModel.ts'));
    const {default: initialize, LocalDuel} = await import(/* @vite-ignore */ String('/src/generated/wasm/brawler_wasm.js'));
    await initialize();
    const game = LocalDuel.skyline_match(await (await fetch('/src/generated/physics.json')).text());
    const stage = document.createElement('div');
    stage.style.cssText = 'width:320px;height:320px';
    document.body.append(stage);
    let scene;
    try {
      const rig = JSON.parse(game.definition_json());
      const map = JSON.parse(game.map_json());
      map.objects = [{kind:'crate', id:2, center:[0,0,0], half:[1.25,1.5,.2]}];
      scene = new FixtureScene(stage, rig, {map, model:await loadFighterModel(), color:'#e040a8'});
      const match = JSON.parse(game.match_json());
      match.phase = 'playing';
      const pose = game.body_states(0);
      // Put the authored rig at the origin so the same camera can see the crate
      // in front of it, to one side, or behind it.
      for (const [i, body] of rig.bodies.entries()) {
        pose.set([...body.position, 0, 0, 0, 1], i * 14);
      }
      // The crate hides one fighter while the center of the group remains clear.
      // A crate across the central camera ray now selects an unobstructed view.
      scene.addOpponent(rig,'#64b7dd',1);
      const peer = pose.slice();
      for (let i=0; i<peer.length; i+=14) peer[i] -= 4;
      const renderer = scene;
      const gl = stage.querySelector('canvas')!.getContext('webgl2')!;
      function draw(x: number, z: number, crateVisible: boolean, fighterVisible = true) {
        match.visible = [fighterVisible,true];
        const crate = new Float32Array([x, 1.75, z, 0, 0, 0, 1, Number(crateVisible)]);
        renderer.renderSkyline([pose,peer], crate, match, 1/60);
        const image = new Uint8Array(gl.drawingBufferWidth * gl.drawingBufferHeight * 4);
        gl.readPixels(0, 0, gl.drawingBufferWidth, gl.drawingBufferHeight, gl.RGBA, gl.UNSIGNED_BYTE, image);
        let count = 0;
        for (let i = 0; i < image.length; i += 4) {
          if (image[i] === 224 && image[i+1] === 64 && image[i+2] === 168) count++;
        }
        return count;
      }
      return {
        behind: draw(0, -2, true), covered: draw(.2, 1.1, true), partial: draw(1.15, 1.1, true),
        retired: draw(.2, 1.1, false), eliminated: draw(.2, 1.1, true, false),
      };
    } finally {
      scene?.dispose();
      game.free();
      stage.remove();
    }
  });
  expect(pixels.covered).toBeGreaterThan(200);
  expect(pixels.partial).toBeGreaterThan(pixels.covered * .15);
  expect(pixels.partial).toBeLessThan(pixels.covered * .85);
  expect([pixels.behind, pixels.retired, pixels.eliminated]).toEqual([0, 0, 0]);
});
