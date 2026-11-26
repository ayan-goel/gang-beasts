import { readFileSync } from 'node:fs';
import { renderToStaticMarkup } from 'react-dom/server';
import { Group } from 'three';
import { expect, it } from 'vitest';
import { MatchFeed } from './MatchFeed';
import OnlineMatch from './OnlineMatch';

const fromRoot = (path: string) => readFileSync(new URL('../../../' + path, import.meta.url));

function fixtureFeed() {
  const feed = new MatchFeed(Uint8Array.of(23,1,0,0,0,1,65,1,1,1,66), () => {});
  feed.receive(fromRoot('tests/fixtures/protocol/full-skyline-v1.bin'));
  return feed;
}

function screen(feed: MatchFeed) {
  const content = {
    map:JSON.parse(fromRoot('assets/maps/skyline/map.json').toString()),
    fighter:JSON.parse(fromRoot('config/character.json').toString()),
    engineHash:'', mapHash:'', model:new Group(),
  };
  return renderToStaticMarkup(<OnlineMatch feed={feed} content={content} exit={() => {}} rematch={undefined}/>);
}

it('renders the acknowledged round and scores in the first screen, before WebGL starts', () => {
  const feed = fixtureFeed();
  const html = screen(feed);
  expect(html).toContain('aria-label="Round number">Round ' + feed.latest!.number);
  expect(html).toContain('aria-label="A score">' + feed.latest!.fighters[0]!.score);
  expect(html).toContain('aria-label="B score">' + feed.latest!.fighters[1]!.score);
});

it.each([
  [2, null, 'Round drawn. No point awarded.'],
  [2, 1, 'B wins the round'],
  [3, 0, 'A wins the match'],
] as const)('announces the authoritative phase %s outcome for winner %s', (phase, winner, label) => {
  const feed = fixtureFeed();
  feed.latest = { ...feed.latest!, phase, winner };
  expect(screen(feed)).toContain(`role="status">${label}</span>`);
});
