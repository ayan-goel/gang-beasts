import { Color, DataTexture, MeshStandardMaterial, NearestFilter, SRGBColorSpace } from 'three';

export function paintSuit(map: DataTexture, team: string, kit: number) {
  const paint = [team, '#361c36', '#f6dfaa', '#657a82', '#324d58', '#344c5b', '#fff8df', '#ffa348'];
  for (let bit = 0; bit < 4; bit++) paint.push((kit + 1) & (1 << bit) ? '#fff8df' : '#361c36');
  for (let bit = 0; bit < 4; bit++) paint.push((kit + 1) & (1 << bit) ? '#f6dfaa' : team);
  const pixels = map.image.data as Uint8Array;
  paint.forEach((value, i) => {
    const hex = new Color(value).getHex();
    pixels.set([hex >>> 16, (hex >>> 8) & 255, hex & 255, 255], i * 4);
  });
  map.needsUpdate = true;
}

export function suitMaterial(team: string, kit: number) {
  const map = new DataTexture(new Uint8Array(16 * 4), 16, 1);
  map.colorSpace = SRGBColorSpace;
  map.magFilter = map.minFilter = NearestFilter;
  paintSuit(map, team, kit);
  return Object.assign(new MeshStandardMaterial({roughness:.46}), {map});
}
