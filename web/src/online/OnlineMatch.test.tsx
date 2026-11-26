import { readFileSync } from 'node:fs';
import { renderToStaticMarkup } from 'react-dom/server';
import { Group } from 'three';
import { expect, it } from 'vitest';
import { MatchFeed } from './MatchFeed';
import OnlineMatch from './OnlineMatch';

const fromRoot = (path: string) => readFileSync(new URL('../../../' + path, import.meta.url));

it('renders the acknowledged round and scores in the first screen, before WebGL starts', () => {
  const feed = new MatchFeed(Uint8Array.of(23,1,0,0,0,1,65,1,1,1,66), () => {});
  feed.receive(fromRoot('tests/fixtures/protocol/full-skyline-v1.bin'));
  const content = {
    map:JSON.parse(fromRoot('assets/maps/skyline/map.json').toString()),
    fighter:JSON.parse(fromRoot('config/character.json').toString()),
    engineHash:'', mapHash:'', model:new Group(),
  };
  const html = renderToStaticMarkup(<OnlineMatch feed={feed} content={content} exit={() => {}} rematch={undefined}/>);
  expect(html).toContain('aria-label="Round number">Round ' + feed.latest!.number);
  expect(html).toContain('aria-label="A score">' + feed.latest!.fighters[0]!.score);
  expect(html).toContain('aria-label="B score">' + feed.latest!.fighters[1]!.score);
});
