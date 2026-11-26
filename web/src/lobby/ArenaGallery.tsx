import { useEffect, useRef, useState } from 'react';

const images = import.meta.glob<string>('./previews/*.webp', {eager:true, import:'default'});
const arenas = Object.entries(images).map(([path, src]) => {
  const id = path.slice('./previews/'.length, -'.webp'.length);
  return {src, name:id.split('-').map(word => word[0]!.toUpperCase() + word.slice(1)).join(' ')};
});
type Arena = typeof arenas[number];

function shuffle(last?: Arena) {
  const deck = [...arenas];
  for (let i = deck.length - 1; i > 0; i--) {
    const j = Math.floor(Math.random() * (i + 1));
    [deck[i], deck[j]] = [deck[j]!, deck[i]!];
  }
  if (deck[0] === last) [deck[0], deck[1]] = [deck[1]!, deck[0]!];
  return deck;
}

export function ArenaGallery() {
  const panel = useRef<HTMLDivElement>(null);
  const [visible, setVisible] = useState(false);
  const [slide, setSlide] = useState(() => {
    const deck = shuffle();
    return {current:deck.shift()!, previous:null as Arena | null, deck};
  });
  useEffect(() => {
    let onScreen = false;
    const update = () => setVisible(onScreen && !document.hidden);
    const observer = new IntersectionObserver(([entry]) => { onScreen = !!entry?.isIntersecting; update(); });
    observer.observe(panel.current!);
    document.addEventListener('visibilitychange', update);
    return () => { observer.disconnect(); document.removeEventListener('visibilitychange', update); };
  }, []);
  useEffect(() => {
    if (!visible) return;
    let active = true;
    let timer: number;
    let deck = [...slide.deck];
    async function prepare() {
      if (!deck.length) deck = shuffle(slide.current);
      const next = deck.shift()!;
      const image = new Image();
      image.src = next.src;
      try {
        // Decode offscreen so slow connections keep the current arena visible until the next is ready.
        await image.decode();
        if (active) timer = window.setTimeout(() => {
          setSlide({current:next, previous:slide.current, deck});
        }, 5500);
      } catch {
        if (active) timer = window.setTimeout(prepare, 1000);
      }
    }
    void prepare();
    return () => { active = false; clearTimeout(timer); };
  }, [slide, visible]);
  return <div className="arena-gallery" ref={panel} role="region" aria-label="Arena previews">
    {[slide.previous, slide.current].map(arena => arena && <figure key={arena.src} aria-hidden={arena !== slide.current}
      className={arena === slide.current && slide.previous ? 'arriving' : undefined}>
      <img src={arena.src} alt="" />
      <figcaption><strong>{arena.name}</strong></figcaption>
    </figure>)}
  </div>;
}
