import { BoxGeometry, Group, Mesh, MeshStandardMaterial } from 'three';

export function fixtureTerrain(boxes: Float32Array = new Float32Array([0, -0.5, 0, 6, 0.5, 6])) {
  const group = new Group();
  for (let i = 0; i < boxes.length; i += 6) {
    const mesh = new Mesh(
      new BoxGeometry(2 * boxes[i + 3]!, 2 * boxes[i + 4]!, 2 * boxes[i + 5]!),
      new MeshStandardMaterial({ color: i === 0 ? '#789a96' : i % 12 ? '#c3ad8b' : '#a38c71', roughness: 0.95 }),
    );
    mesh.position.set(boxes[i]!, boxes[i + 1]!, boxes[i + 2]!);
    mesh.receiveShadow = true;
    group.add(mesh);
  }
  return group;
}
