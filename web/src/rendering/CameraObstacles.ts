import { Box3, Object3D, Quaternion, Ray, Vector3 } from 'three';

interface Obstacle { half: Vector3; pose: Object3D }

export class CameraObstacles {
  private readonly boxes: Obstacle[] = [];
  private readonly inverse = new Quaternion();
  private readonly bounds = new Box3();
  private readonly ray = new Ray();
  private readonly hit = new Vector3();
  private readonly up = new Vector3();
  private readonly result = {distance:0, top:-Infinity};

  add(half: [number,number,number], pose: Object3D) {
    this.boxes.push({half:new Vector3(...half),pose});
  }

  fixed(center: [number,number,number], half: [number,number,number]) {
    const pose = new Object3D();
    pose.position.set(...center);
    this.add(half,pose);
  }

  // The result is reused for successive pitch probes on the render thread.
  cast(origin: Vector3, direction: Vector3, length: number, radius: number) {
    const result = this.result;
    result.distance = length;
    result.top = -Infinity;
    for (const {half,pose} of this.boxes) {
      if (!pose.visible) continue;
      this.inverse.copy(pose.quaternion).invert();
      this.ray.origin.copy(origin).sub(pose.position).applyQuaternion(this.inverse);
      this.ray.direction.copy(direction).applyQuaternion(this.inverse);
      // Expanding each local face conservatively includes a lens sphere grazing a rotated corner.
      this.bounds.max.copy(half).addScalar(radius);
      this.bounds.min.copy(this.bounds.max).negate();
      let distance = 0;
      if (!this.bounds.containsPoint(this.ray.origin)) {
        if (!this.ray.intersectBox(this.bounds,this.hit)) continue;
        distance = this.hit.distanceTo(this.ray.origin);
      }
      if (distance > result.distance) continue;
      this.up.set(0,1,0).applyQuaternion(this.inverse);
      const h = this.bounds.max;
      const top = pose.position.y + Math.abs(this.up.x)*h.x + Math.abs(this.up.y)*h.y + Math.abs(this.up.z)*h.z;
      result.top = distance === result.distance ? Math.max(result.top,top) : top;
      result.distance = distance;
    }
    return result;
  }
}
