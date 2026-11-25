import {expect,test} from '@playwright/test';

test('walking WASM validates intent atomically and preserves its variant and owned diagnostics', async({page})=>{
  await page.goto('/'); await expect(page.getByRole('status')).toHaveText('Simulation ready');
  const result = await page.evaluate(async()=>{
    const {default:init,LocalRagdoll} = await import(/* @vite-ignore */ String('/src/generated/wasm/brawler_wasm.js')); await init();
    const config = await (await fetch('/src/generated/physics.json')).text();
    const rig = LocalRagdoll.locomotion(config), reference = LocalRagdoll.locomotion(config);
    rig.set_movement(1,0,false); reference.set_movement(1,0,false);
    const rejected=[];
    for(const values of [['1',0,false],[null,0,false],[NaN,0,false],[0,Infinity,false],[Number.MAX_VALUE,0,false],[0,0,'false'],[0,0,1]]) {
      try { rig.set_movement(...values); rejected.push(false); } catch { rejected.push(true); }
    }
    for(let step=0;step<60;step++) {rig.step();reference.step();}
    const unchanged = Array.from(rig.body_states()), expected = Array.from(reference.body_states());
    rig.set_movement(0,0,false); rig.step();
    const owned = [rig.movement_state(), rig.pose_targets(), rig.stopping_moment(), rig.support_moment()] as Float32Array[];
    const retained = owned.map(v=>Array.from(v));
    rig.set_enabled(false);
    rig.configure(JSON.stringify({schema_version:1,gravity_y:-6,solver_iterations:12}));
    const configured = {state:Array.from(rig.movement_state()),moment:Array.from(rig.stopping_moment()),support:Array.from(rig.support_moment()),enabled:rig.controller_state()[0],tick:String(rig.tick()),poses:Array.from(rig.pose_targets())};
    rig.reset(); rig.free(); reference.free();
    const unsupported=[];
    for(const fixture of [new LocalRagdoll(config), LocalRagdoll.standing(config)]) {
      let error=false; try {fixture.set_movement(1,0,false);}catch{error=true;}
      unsupported.push([fixture.movement_state().length,fixture.stopping_moment().length,fixture.support_moment().length,error]);fixture.free();
    }
    return {rejected,unchanged,expected,retained,afterFree:owned.map(v=>Array.from(v)),configured,unsupported};
  });
  expect(result.rejected).toEqual(Array(7).fill(true)); expect(result.unchanged).toEqual(result.expected);
  expect(Math.hypot(...result.retained[2]!)).toBeGreaterThan(1);
  expect(result.retained).toEqual(result.afterFree); expect(result.configured.tick).toBe('0'); expect(result.configured.enabled).toBe(0);
  expect(result.configured.state).toEqual(Array(11).fill(0));
  expect(result.configured.moment).toEqual([0,0,0]);
  expect(result.retained[3]).toEqual([0,0,0]);
  expect(result.configured.support).toEqual([0,0,0]);
  expect(result.configured.poses).toEqual(Array.from({length:14},()=>[0,0,0,1]).flat());
  expect(result.unsupported).toEqual([[0,0,0,true],[0,0,0,true]]);
});
