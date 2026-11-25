import { PerspectiveCamera, Vector3 } from 'three';

const direction = new Vector3(0.18, 0.85, 1).normalize();
const right = new Vector3().crossVectors(new Vector3(0, 1, 0), direction).normalize();
const up = new Vector3().crossVectors(direction, right);
export function cameraRelativeMovement(x: number, z: number): [number, number] {
  return [right.x*x - right.z*z, right.z*x + right.x*z];
}

export class GroupCamera {
  private readonly focus = new Vector3();
  private readonly desired = new Vector3();
  private readonly point = new Vector3();
  private distance = 0;
  update(camera: PerspectiveCamera, players: Float32Array[], overview: boolean, dt: number) {
    this.desired.set(0, 0, 0);
    if (!overview && players.length) {
      for (const p of players) this.desired.add(this.point.fromArray(p, 28));
      this.desired.divideScalar(players.length);
      this.desired.y = Math.max(0.8, this.desired.y);
    }
    const weight = this.distance === 0 ? 1 : 1 - Math.exp(-6 * Math.min(dt, 0.12));
    this.focus.lerp(this.desired, weight);
    const slope = Math.tan(camera.fov * Math.PI / 360);
    let required = 4.5;
    const include = (x: number, y: number, z: number, margin: number) => {
      const p = this.point.set(x, y, z).sub(this.focus);
      required = Math.max(required, p.dot(direction) + Math.max(
        (Math.abs(p.dot(right)) + margin) / (slope * camera.aspect * 0.83),
        (Math.abs(p.dot(up)) + margin) / (slope * 0.76)));
    };
    if (overview || !players.length) {
      for (const x of [-12, 12]) for (const z of [-10, 10]) include(x, 0, z, 0.2);
    } else for (const player of players) {
      for (let i = 0; i < player.length; i += 14) include(player[i]!, player[i + 1]!, player[i + 2]!, 0.25);
    }
    // Expand immediately to keep separated players in frame. Zoom back in gradually.
    this.distance = Math.max(required, this.distance + (required - this.distance) * weight);
    camera.position.copy(this.focus).addScaledVector(direction, this.distance);
    camera.lookAt(this.focus);
  }
}
