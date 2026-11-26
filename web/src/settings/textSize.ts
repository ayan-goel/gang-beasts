import { storedChoice } from './storedChoice';

export const textSizes = ['100', '125', '150', '200'] as const;
export const textSize = storedChoice('skyline.display.v1.text-percent', textSizes, '100');

export function chooseTextSize(value: string) {
  const size = textSizes.find(size => size === value);
  if (size) textSize.choose(size);
}
