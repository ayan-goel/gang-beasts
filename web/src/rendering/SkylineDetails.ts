import { BoxGeometry, BufferGeometry, Color, CylinderGeometry, SphereGeometry, TorusGeometry, Float32BufferAttribute, Mesh, MeshStandardMaterial, Quaternion, Vector3, type Object3D } from 'three';
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

  oval(size: Triple, position: Triple, paint: number) {
    this.paint(new SphereGeometry(1, 12, 8).scale(...size).translate(...position), paint);
  }

  hoop(radius: number, tube: number, position: Triple, paint: number, flat = false) {
    const geometry = new TorusGeometry(radius, tube, 5, 32);
    if (flat) geometry.rotateX(-Math.PI / 2);
    this.paint(geometry.translate(...position), paint);
  }

  fin(width: number, height: number, thickness: number, position: Triple, paint: number, yaw = 0) {
    const geometry = new CylinderGeometry(0, 1, height, 3)
      .scale(width, 1, thickness).rotateY(yaw).translate(...position);
    this.paint(geometry, paint);
  }

  leaf(start: Triple, arch: Triple, end: Triple, halfWidth: number, paint: number) {
    const dx = end[0]-start[0], dz = end[2]-start[2], length = Math.hypot(dx,dz) || 1;
    const px = -dz*halfWidth/length, pz = dx*halfWidth/length;
    const verts = [...start, arch[0]+px,arch[1]-.065,arch[2]+pz, ...arch,
      arch[0]-px,arch[1]-.065,arch[2]-pz, ...end];
    const face = [0,1,2,0,2,3,1,4,2,2,4,3];
    const geometry = new BufferGeometry();
    geometry.setAttribute('position',new Float32BufferAttribute([...verts,...verts],3));
    geometry.setAttribute('uv',new Float32BufferAttribute(new Float32Array(20),2));
    geometry.setIndex([...face,...[...face].reverse().map(i => i+5)]);
    geometry.computeVertexNormals();
    this.paint(geometry,paint);
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
