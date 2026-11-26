import { Color, DataTexture, MeshStandardMaterial, NearestFilter, SRGBColorSpace } from 'three';

export function suitMaterial(team: string, kit: number) {
  const paint = [team, '#263945', '#f6dfaa', '#657a82', '#324d58', '#344c5b', '#fff6d8', '#ffa348'];
  for (let bit = 0; bit < 4; bit++) paint.push((kit + 1) & (1 << bit) ? '#fff6d8' : '#263945');
  for (let bit = 0; bit < 4; bit++) paint.push((kit + 1) & (1 << bit) ? '#f6dfaa' : team);
  const pixels = new Uint8Array(16 * 4);
  paint.forEach((value, i) => {
    const hex = new Color(value).getHex();
    pixels.set([hex >>> 16, (hex >>> 8) & 255, hex & 255, 255], i * 4);
  });
  const map = new DataTexture(pixels, 16, 1);
  map.colorSpace = SRGBColorSpace;
  map.magFilter = map.minFilter = NearestFilter;
  map.needsUpdate = true;
  return new MeshStandardMaterial({map, roughness:.78});
}
