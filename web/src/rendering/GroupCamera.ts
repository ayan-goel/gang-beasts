import { PerspectiveCamera, Vector3 } from 'three';

import { CameraFit } from './CameraFit';
import type { SkylineMap } from './SkylineView';
import type { CameraObstacles } from './CameraObstacles';
import type { RigDefinition } from '../game/rigDefinition';

const defaultDirection = new Vector3(0.18, 0.85, 1).normalize();
const vertical = new Vector3(0, 1, 0);
const increment = Math.PI / 4;

const normalPitch = Math.asin(defaultDirection.y);
const alternativePitches = [50,30,60,20,70,12,80].map(degrees => degrees * Math.PI / 180);

export class GroupCamera {
  reducedMotion = false;
  private readonly direction = defaultDirection.clone();
  private readonly right = new Vector3().crossVectors(vertical, defaultDirection).normalize();
  private readonly up = new Vector3().crossVectors(defaultDirection, this.right);
  private angle = 0;
  private touring = false;
  private headingChanged = false;
  private preset = 0;
  private readonly fit: CameraFit;
  private readonly pivot = new Vector3();
  private pitch = normalPitch;
  private lift = 0;

  constructor(bodies: RigDefinition['bodies'], visualRadii?: number[], private obstacles?: CameraObstacles) {
    this.fit = new CameraFit(visualRadii ?? bodies.map(({shape}) => {
      switch (shape.kind) {
        case 'sphere': return shape.radius;
        case 'capsule': return shape.half_height + shape.radius;
        case 'rounded_box': return Math.hypot(...shape.half_extents) + shape.border_radius;
      }
    }));
  }

  setArena(obstacles: CameraObstacles, map: SkylineMap) {
    this.obstacles = obstacles;
    this.headingChanged = this.touring !== !!map.tour;
    this.touring = !!map.tour;
    const decks = map.objects.filter(obj => obj.kind === 'deck' || obj.kind === 'panel');
    if (decks.length && map.tour) {
      this.fit.overviewBounds = [
        Math.min(...decks.map(obj => obj.center[0]-obj.half[0])),
        Math.max(...decks.map(obj => obj.center[0]+obj.half[0])),
        Math.min(...decks.map(obj => obj.center[2]-obj.half[2])),
        Math.max(...decks.map(obj => obj.center[2]+obj.half[2])),
      ];
    } else this.fit.overviewBounds = [-12,12,-10,10];
    // Keep the controller's orbit reference while snapping away from the previous map's collision volumes.
    this.distance = this.lift = 0;
    this.pitch = this.touring ? Math.PI / 9 : normalPitch;
  }

  turn(direction: -1 | 1) {
    this.preset = (this.preset + direction + 8) % 8;
  }

  movement(x: number, z: number): [number, number] {
    return [this.right.x*x - this.right.z*z, this.right.z*x + this.right.x*z];
  }

  private readonly focus = new Vector3();
  private readonly desired = new Vector3();
  private readonly point = new Vector3();
  private distance = 0;
  update(camera: PerspectiveCamera, players: Float32Array[], overview: boolean, dt: number, owner?: Float32Array) {
    const previousAngle = this.angle;
    const frameTime = Math.min(.12,Math.max(0,dt));
    const target = this.preset * increment;
    const delta = Math.atan2(Math.sin(target - this.angle), Math.cos(target - this.angle));
    this.angle = this.reducedMotion || Math.abs(delta) < 0.00001 ? target
      : this.angle + delta * (1 - Math.exp(-8 * frameTime));
    const {direction, right, up} = this;
    direction.copy(defaultDirection).applyAxisAngle(vertical, this.angle + (this.touring ? .4 : 0));
    right.crossVectors(vertical, direction).normalize();
    this.desired.set(0, 0, 0);
    if (!overview && players.length) {
      let total = 0;
      for (const p of players) {
        const weight = p === owner ? 1.35 : 1;
        this.desired.addScaledVector(this.point.fromArray(p, 28), weight);
        total += weight;
      }
      this.desired.divideScalar(total);
      this.desired.y = Math.max(0.8, this.desired.y);
    }
    const weight = this.distance === 0 ? 1 : 1 - Math.exp(-6 * frameTime);
    let panWeight = weight;
    if (this.reducedMotion && this.distance > 0 && !overview && players.length > 0) {
      // Allow body sway inside a 75 cm horizontal and 25 cm vertical margin.
      // Tracking just the excess avoids a camera jump when a fighter crosses it.
      const offset = this.point.subVectors(this.desired, this.focus);
      offset.y *= 3;
      const span = offset.length();
      panWeight *= span > .75 ? 1 - .75 / span : 0;
    }
    this.focus.lerp(this.desired, panWeight);
    const tangent = Math.tan(camera.fov * Math.PI / 360);
    const verticalSlope = tangent * .70;
    const radius = Math.max(.15,camera.near * Math.hypot(1,tangent,tangent*camera.aspect));
    this.pivot.copy(this.focus);
    this.pivot.y += this.lift*(1-weight);
    let blockedTop = -Infinity, inside = false;
    const tryView = (pitch: number) => {
      const horizontal = Math.cos(pitch);
      direction.set(-right.z*horizontal,Math.sin(pitch),right.x*horizontal);
      up.crossVectors(direction,right);
      this.fit.measure(this.pivot,direction,right,up,verticalSlope,camera.aspect,players,overview);
      const required = this.fit.distance;
      let distance = this.framingDistance(required, weight);
      const hit = this.obstacles?.cast(this.pivot,direction,distance,radius);
      if (hit && hit.top !== -Infinity) {
        blockedTop = hit.top;
        inside = hit.distance === 0;
        distance = hit.distance - .01;
        if (distance < required) return false;
      }
      this.pitch = pitch;
      this.distance = distance;
      return true;
    };
    const restingPitch = this.touring ? (overview ? 20 : 32) * Math.PI/180 : normalPitch;
    const relaxed = this.pitch + (restingPitch-this.pitch)*weight;
    while (!tryView(relaxed)) {
      // Keep the last clear aim when easing down would dip back into the same obstacle.
      if (this.pivot.y < this.focus.y+this.lift) {
        this.pivot.y = this.focus.y+this.lift;
        if (tryView(relaxed)) break;
      }
      const lowObstacle = blockedTop + .01 <= this.focus.y + .65;
      if (!inside && !lowObstacle) {
        if (relaxed !== this.pitch && tryView(this.pitch)) break;
        if (alternativePitches.some(tryView)) break;
      }
      // Above the expanded top, every permitted upward ray clears this solid. Each retry clears another.
      this.pivot.y = blockedTop + .01;
      inside = false;
    }
    this.lift = this.pivot.y-this.focus.y;
    const far = Math.ceil(this.distance + this.fit.behind + 1);
    if (far > camera.far) { camera.far = far; camera.updateProjectionMatrix(); }
    camera.position.copy(this.pivot).addScaledVector(direction, this.distance);
    camera.lookAt(this.pivot);
    const turned = this.headingChanged || previousAngle !== this.angle;
    this.headingChanged = false;
    return turned;
  }

  private framingDistance(required: number, weight: number) {
    if (this.reducedMotion) {
      // Spare framing room absorbs limb motion. Zoom in only after the group occupies
      // substantially less space, while newly separated fighters still fit immediately.
      const comfortable = required * 1.12;
      if (this.distance < required) return comfortable;
      if (this.distance <= required * 1.25) return this.distance;
      return this.distance + (comfortable - this.distance) * weight;
    }
    return Math.max(required, this.distance + (required - this.distance) * weight);
  }
}
