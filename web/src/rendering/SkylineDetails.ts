import { BoxGeometry, Color, Float32BufferAttribute, Mesh, MeshStandardMaterial, type Object3D } from 'three';
import { mergeGeometries } from 'three/addons/utils/BufferGeometryUtils.js';

type Triple = [number, number, number];

export class SkylineDetails {
  private readonly boxes: BoxGeometry[] = [];

  add(size: Triple, position: Triple, paint: number) {
    const geometry = new BoxGeometry(...size).translate(...position);
    const count = geometry.getAttribute('position').count;
    const rgb = new Float32Array(count * 3);
    const color = new Color(paint);
    for (let i = 0; i < count; i++) color.toArray(rgb, i*3);
    geometry.setAttribute('color', new Float32BufferAttribute(rgb, 3));
    this.boxes.push(geometry);
  }

  attach(parent: Object3D) {
    if (!this.boxes.length) return;
    // Details fixed to one parent share a draw. Moving props and warning panels retain separate transforms.
    const geometry = mergeGeometries(this.boxes);
    if (!geometry) throw new Error('Invalid Skyline detail geometry');
    const mesh = new Mesh(geometry, new MeshStandardMaterial({vertexColors:true, roughness:.78}));
    mesh.castShadow = mesh.receiveShadow = true;
    parent.add(mesh);
    this.boxes.length = 0;
  }
}
