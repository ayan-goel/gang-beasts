import { expect, test } from '@playwright/test';

test('every arena opens on the same canvas with both fighters and its own physical layout', async ({page}, info) => {
  test.setTimeout(180000);
  await page.setViewportSize({width:800,height:500});
  await page.goto('/');
  const errors: string[] = [];
  page.on('pageerror', e => errors.push(e.message));
  page.on('response', response => {
    if (response.url().includes('/rendering/tour/') && !response.ok() && response.status() !== 304) errors.push(response.url());
  });
  const gallery = await page.evaluateHandle(async () => {
    const {FixtureScene} = await import(/* @vite-ignore */ String('/src/rendering/FixtureScene.ts'));
    const {loadFighterModel} = await import(/* @vite-ignore */ String('/src/rendering/fighter/loadFighterModel.ts'));
    const wasm = await import(/* @vite-ignore */ String('/src/generated/wasm/brawler_wasm.js'));
    await wasm.default();
    const config = await (await fetch('/src/generated/physics.json')).text();
    const openingSeeds = new Map<string, number>();
    for (let seed=1;seed<1000 && openingSeeds.size<22;seed++) {
      const id = JSON.parse(wasm.tour_map(seed,1)).id;
      if (!openingSeeds.has(id)) openingSeeds.set(id, seed);
    }
    const stage = document.createElement('div');
    stage.style.cssText='position:fixed;inset:0;z-index:100;background:#351d37';
    document.body.append(stage);
    const first = wasm.LocalDuel.tour_match(config,4,3);
    const rig = JSON.parse(first.definition_json());
    const scene = new FixtureScene(stage,rig,{map:JSON.parse(first.map_json()),model:await loadFighterModel()});
    scene.addOpponent(rig);
    first.free();
    return {
      ids:Array.from(openingSeeds.keys()).sort(),
      async show(id: string) {
        const game = wasm.LocalDuel.tour_match(config, openingSeeds.get(id), 3);
        try {
          const map = JSON.parse(game.map_json());
          scene.setArena(map);
          const match = JSON.parse(game.match_json());
          const players = [game.body_states(0),game.body_states(1)];
          const props = game.arena_poses();
          const draw = () => scene.renderSkyline(players,props,match,1/60);
          for(let i=0;i<4;i++) {draw(); await new Promise(r=>requestAnimationFrame(r));}
          draw();
          const gl = stage.querySelector('canvas')!.getContext('webgl2')!;
          return {id:map.id, count:props.length/8, solids:map.objects.length, error:gl.getError(), canvases:stage.querySelectorAll('canvas').length};
        } finally {game.free();}
      },
      close(){scene.dispose();stage.remove();},
    };
  });
  try {
    const ids = await gallery.evaluate(g=>g.ids);
    expect(ids).toHaveLength(22);
    for (const id of ids) {
      const frame = await gallery.evaluate((g,id)=>g.show(id),id);
      expect(frame).toMatchObject({id,count:frame.solids,canvases:1,error:0});
      await page.locator('canvas').screenshot({path:info.outputPath(id+'.png')});
    }
    expect(errors).toEqual([]);
  } finally {await gallery.evaluate(g=>g.close());await gallery.dispose();}
});
