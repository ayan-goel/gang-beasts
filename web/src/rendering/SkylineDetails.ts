import { BoxGeometry, BufferGeometry, Color, CylinderGeometry, Float32BufferAttribute, Mesh, MeshStandardMaterial, Quaternion, Vector3, type Object3D } from 'three';
import { mergeGeometries } from 'three/addons/utils/BufferGeometryUtils.js';

type Triple = [number, number, number];

export class SkylineDetails {
  private readonly parts: BufferGeometry[] = [];

  add(size: Triple, position: Triple, paint: number, yaw = 0) {
    this.paint(new BoxGeometry(...size).rotateY(yaw).translate(...position), paint);
  }

  drum(top: number, base: number, height: number, position: Triple, paint: number, sides = 16) {
    this.paint(new CylinderGeometry(top, base, height, sides).translate(...position), paint);
  }

  rod(a: Triple, b: Triple, radius: number, paint: number) {
    const start = new Vector3(...a), end = new Vector3(...b);
    const direction = end.clone().sub(start);
    const rotation = new Quaternion().setFromUnitVectors(new Vector3(0,1,0), direction.clone().normalize());
    const center = start.lerp(end, .5);
    this.paint(new CylinderGeometry(radius,radius,direction.length(),6).applyQuaternion(rotation)
      .translate(center.x,center.y,center.z),paint);
  }

  private paint(geometry: BufferGeometry, paint: number) {
    const count = geometry.getAttribute('position').count;
    const rgb = new Float32Array(count * 3);
    const color = new Color(paint);
    for (let i = 0; i < count; i++) color.toArray(rgb, i*3);
    geometry.setAttribute('color', new Float32BufferAttribute(rgb, 3));
    this.parts.push(geometry);
  }

  attach(parent: Object3D) {
    if (!this.parts.length) return;
    // Batch decoration per physical object so a falling panel carries its own trim.
    const geometry = mergeGeometries(this.parts);
    if (!geometry) throw new Error('Invalid Skyline detail geometry');
    const mesh = new Mesh(geometry, new MeshStandardMaterial({vertexColors:true, roughness:.73}));
    mesh.castShadow = mesh.receiveShadow = true;
    parent.add(mesh);
    this.parts.length = 0;
  }
}
