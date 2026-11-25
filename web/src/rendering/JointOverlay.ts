import * as THREE from 'three';

export class JointOverlay {
  readonly root = new THREE.Group();
  private readonly positions: Float32Array;
  private readonly anchors: Float32Array;
  private readonly lines: THREE.LineSegments;
  private readonly points: THREE.Points;
  private readonly origin = new THREE.Vector3();
  private readonly endpoint = new THREE.Vector3();
  private readonly rotation = new THREE.Quaternion();

  constructor(private readonly count: number) {
    this.positions = new Float32Array(count * 42);
    this.anchors = new Float32Array(count * 6);
    const colors = new Float32Array(this.positions.length);
    for (let joint = 0; joint < count; joint++) {
      const palette = [0xffffff, 0xffcccc, 0xcfffd8, 0xc4ddff, 0xff5555, 0x55ff7b, 0x559fff];
      palette.forEach((hex, segment) => {
        const color = new THREE.Color(hex);
        color.toArray(colors, joint * 42 + segment * 6);
        color.toArray(colors, joint * 42 + segment * 6 + 3);
      });
    }
    const geometry = new THREE.BufferGeometry();
    geometry.setAttribute('position', new THREE.BufferAttribute(this.positions, 3).setUsage(THREE.DynamicDrawUsage));
    geometry.setAttribute('color', new THREE.BufferAttribute(colors, 3));
    this.lines = new THREE.LineSegments(geometry, new THREE.LineBasicMaterial({ vertexColors: true, depthTest: false }));
    const pointsGeometry = new THREE.BufferGeometry();
    pointsGeometry.setAttribute('position', new THREE.BufferAttribute(this.anchors, 3).setUsage(THREE.DynamicDrawUsage));
    this.points = new THREE.Points(pointsGeometry, new THREE.PointsMaterial({ color: '#fff2ca', size: 0.045, depthTest: false }));
    for (const object of [this.lines, this.points]) { object.frustumCulled = false; object.renderOrder = 3; }
    this.root.add(this.lines, this.points);
  }

  update(states?: Float32Array): void {
    this.root.visible = !!states;
    if (!states) return;
    if (states.length !== this.count * 17) throw new Error('Invalid joint-frame buffer');
    for (let joint = 0; joint < this.count; joint++) {
      const input = joint * 17;
      const output = joint * 42;
      this.positions.set(states.subarray(input + 2, input + 8), output);
      this.anchors.set(states.subarray(input + 2, input + 8), joint * 6);
      for (let side = 0; side < 2; side++) {
        this.origin.fromArray(states, input + 2 + side * 3);
        this.rotation.fromArray(states, input + 8 + side * 4);
        for (let axis = 0; axis < 3; axis++) {
          const offset = output + 6 + side * 18 + axis * 6;
          this.origin.toArray(this.positions, offset);
          this.endpoint.set(0, 0, 0).setComponent(axis, 0.15).applyQuaternion(this.rotation).add(this.origin);
          this.endpoint.toArray(this.positions, offset + 3);
        }
      }
    }
    this.lines.geometry.getAttribute('position').needsUpdate = true;
    this.points.geometry.getAttribute('position').needsUpdate = true;
  }
}
