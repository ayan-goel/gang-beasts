import { expect, it } from 'vitest';
import rig from '../../../config/character.json';
import { readFileSync } from 'node:fs';
import type { SkylineMap } from '../rendering/SkylineView';
import { onlineGripLabels } from './gripLabels';

const names = ['Mint','Blue','A','B','C','D','E','Rose'];
const map: SkylineMap = JSON.parse(readFileSync(new URL('../../../assets/maps/skyline/map.json', import.meta.url),'utf8'));
it('identifies both hands independently using match roster order and anatomical part indices', () => {
  expect(onlineGripLabels([113,18],names,rig.bodies,map))
    .toEqual(['Left hand on Rose’s head','Right hand on Blue’s chest']);
  expect(onlineGripLabels([0,127],names,rig.bodies,map)).toEqual(['Right hand on Rose’s right foot']);
  expect(onlineGripLabels([0,0],names,rig.bodies,map)).toEqual([]);
});

it('distinguishes deck, walkways and movable scenery by map object ID', () => {
  expect(onlineGripLabels([65537,65544],names,rig.bodies,map)).toEqual(['Left hand on roof edge','Right hand on walkway edge']);
  expect(onlineGripLabels([65538,65543],names,rig.bodies,map)).toEqual(['Left hand on vent','Right hand on crate']);
});
