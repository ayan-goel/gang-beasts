export interface CameraPosition { pan: number; distance: number }

export function distanceGain(distance: number) {
  return 8 / Math.max(8, distance);
}
