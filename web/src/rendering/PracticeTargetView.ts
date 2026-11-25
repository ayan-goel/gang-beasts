import * as THREE from 'three';
import { RoundedBoxGeometry } from 'three/addons/geometries/RoundedBoxGeometry.js';

export class PracticeTargetView {
  readonly root = new THREE.Group();
  private readonly pad = new THREE.Group();
  private readonly pivot = new THREE.Mesh(
    new THREE.SphereGeometry(0.045, 12, 8),
    new THREE.MeshStandardMaterial({ color: '#e6d8bb', metalness: 0.35, roughness: 0.48 }),
  );
  private readonly tether = new THREE.Mesh(
    new THREE.CylinderGeometry(0.012, 0.012, 1, 8),
    new THREE.MeshStandardMaterial({ color: '#dbc7a4' }),
  );
  private readonly up = new THREE.Vector3(0, 1, 0);
  private readonly top = new THREE.Vector3();
  private readonly direction = new THREE.Vector3();

  constructor(state: Float32Array) {
    const [halfX, halfY, halfZ] = [state[7]!, state[8]!, state[9]!];
    const cushion = new THREE.Mesh(new RoundedBoxGeometry(2 * halfX, 2 * halfY, 2 * halfZ, 2, 0.022),
      new THREE.MeshStandardMaterial({ color: '#265b81', roughness: 0.85 }));
    cushion.castShadow = true;
    const markings = new THREE.MeshStandardMaterial({ color: '#f9d999', roughness: 0.85 });
    const ring = new THREE.RingGeometry(0.052, 0.067, 24);
    for (const side of [-1, 1]) {
      for (const face of [-1, 1]) {
        const mark = new THREE.Mesh(ring, markings);
        mark.position.set(side * 0.67 * halfX, 0, face * (halfZ + 0.001));
        if (face < 0) mark.rotation.y = Math.PI;
        this.pad.add(mark);
      }
    }
    this.pad.add(cushion);
    this.root.add(this.pad, this.tether, this.pivot);
    this.update(state);
  }

  update(state: Float32Array): void {
    this.pad.position.fromArray(state);
    this.pad.quaternion.fromArray(state, 3);
    this.pivot.position.fromArray(state, 10);
    this.top.set(0, state[8]!, 0).applyQuaternion(this.pad.quaternion).add(this.pad.position);
    this.direction.subVectors(this.pivot.position, this.top);
    this.tether.scale.y = this.direction.length();
    this.tether.position.copy(this.top).lerp(this.pivot.position, 0.5);
    this.tether.quaternion.setFromUnitVectors(this.up, this.direction.normalize());
  }
}
