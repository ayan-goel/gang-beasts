import * as THREE from 'three';

export class ContactNormals extends THREE.LineSegments<THREE.BufferGeometry, THREE.LineBasicMaterial> {
  constructor() {
    super(new THREE.BufferGeometry(), new THREE.LineBasicMaterial({ color: '#28fff0', depthTest: false }));
    this.frustumCulled = false;
    this.renderOrder = 4;
    this.visible = false;
  }

  update(contacts?: Float32Array): void {
    const count = (contacts?.length ?? 0) / 7;
    if (!Number.isInteger(count)) throw new Error('Invalid limb contact buffer');
    this.visible = count > 0;
    if (!contacts || !count) return;
    let vertices = this.geometry.getAttribute('position');
    if (!vertices || vertices.count < count * 2) {
      // Dispose the old GPU buffer before replacing its attribute at a new high-water mark.
      this.geometry.dispose();
      vertices = new THREE.BufferAttribute(new Float32Array(count * 6), 3).setUsage(THREE.DynamicDrawUsage);
      this.geometry.setAttribute('position', vertices);
    }
    for (let i = 0; i < count; i++) {
      const offset = i * 7;
      const x = contacts[offset]!, y = contacts[offset + 1]!, z = contacts[offset + 2]!;
      vertices.setXYZ(2 * i, x, y, z);
      vertices.setXYZ(2 * i + 1, x + 0.2 * contacts[offset + 3]!, y + 0.2 * contacts[offset + 4]!, z + 0.2 * contacts[offset + 5]!);
    }
    vertices.needsUpdate = true;
    this.geometry.setDrawRange(0, 2 * count);
  }
}
