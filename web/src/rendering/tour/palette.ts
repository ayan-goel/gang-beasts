import colors from './palettes.json';

export type StagePalette = typeof colors[keyof typeof colors];

export function stagePalette(id: string): StagePalette {
  const palette = colors[id as keyof typeof colors];
  if (!palette) throw new Error('Missing arena artwork for ' + id);
  return palette;
}
