import { Vector3 } from 'three';

export class CameraFit {
  private readonly point = new Vector3();
  distance = 0;
  behind = 0;
  overviewBounds: readonly [number, number, number, number] = [-12,12,-10,10];

  constructor(private readonly radii: readonly number[]) {}

  measure(focus: Vector3, direction: Vector3, right: Vector3, up: Vector3, verticalSlope: number, aspect: number,
      players: Float32Array[], overview: boolean) {
    const horizontalSlope = verticalSlope * aspect;
    const horizontalPlane = Math.hypot(1,1/horizontalSlope);
    const verticalPlane = Math.hypot(1,1/verticalSlope);
    this.distance = 4.5;
    this.behind = 0;
    const include = (x: number, y: number, z: number, radius: number) => {
      const p = this.point.set(x,y,z).sub(focus);
      const depth = p.dot(direction);
      // Each inset plane must clear the whole limb sphere, including its extent toward the camera.
      this.distance = Math.max(this.distance, depth + Math.max(
        Math.abs(p.dot(right))/horizontalSlope + radius*horizontalPlane,
        Math.abs(p.dot(up))/verticalSlope + radius*verticalPlane));
      this.behind = Math.max(this.behind, radius-depth);
    };
    if (overview || !players.length) {
      const [left,right,back,front] = this.overviewBounds;
      for (const x of [left,right]) for (const z of [back,front]) include(x,0,z,.2);
    } else for (const player of players) {
      for (let i=0; i<player.length; i+=14) include(player[i]!,player[i+1]!,player[i+2]!,this.radii[i/14]!);
    }
  }
}
