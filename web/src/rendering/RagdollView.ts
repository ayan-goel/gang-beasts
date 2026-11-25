import * as THREE from 'three';
import { RoundedBoxGeometry } from 'three/addons/geometries/RoundedBoxGeometry.js';
import type { BodyShape, RigDefinition } from '../game/rigDefinition';

export function geometryForBody(shape: BodyShape): THREE.BufferGeometry {
  switch (shape.kind) {
    case 'sphere': return new THREE.SphereGeometry(shape.radius, 24, 16);
    case 'capsule': return new THREE.CapsuleGeometry(shape.radius, shape.half_height * 2, 8, 16);
    case 'rounded_box': {
      // Rapier adds the border outside the cuboid; Three takes total dimensions.
      const size = shape.half_extents.map(half => 2 * (half + shape.border_radius));
      return new RoundedBoxGeometry(size[0], size[1], size[2], 3, shape.border_radius);
    }
  }
}

export class RagdollView {
  readonly root = new THREE.Group();
  private readonly meshes: THREE.Mesh<THREE.BufferGeometry, THREE.MeshStandardMaterial>[];

  constructor(definition: RigDefinition, color = '#f58465') {
    this.meshes = definition.bodies.map(body => {
      const mesh = new THREE.Mesh(geometryForBody(body.shape), new THREE.MeshStandardMaterial({
        color: body.part.includes('hand') || body.part.includes('foot') ? '#bad8d2' : color, roughness: 0.7,
      }));
      mesh.name = body.part;
      mesh.castShadow = true;
      mesh.receiveShadow = true;
      this.root.add(mesh);
      return mesh;
    });
  }

  update(states: Float32Array, selectedBody: number): void {
    if (states.length !== this.meshes.length * 14) throw new Error('Invalid ragdoll body buffer');
    this.meshes.forEach((mesh, index) => {
      const offset = index * 14;
      mesh.position.fromArray(states, offset);
      mesh.quaternion.fromArray(states, offset + 3);
      mesh.material.emissive.setHex(index === selectedBody ? 0x593320 : 0);
    });
  }
}
