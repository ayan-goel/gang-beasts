import { GreaterDepth, MeshBasicMaterial, SkinnedMesh } from 'three';

export function addOccludedSilhouette(fighter: SkinnedMesh, team: string) {
  const material = new MeshBasicMaterial({
    color: team, depthFunc: GreaterDepth, depthWrite: false, fog: false, toneMapped: false,
  });
  material.onBeforeCompile = shader => {
    shader.fragmentShader = shader.fragmentShader.replace('#include <alphatest_fragment>', `
      if (mod(gl_FragCoord.x + gl_FragCoord.y, 6.0) < 2.0) discard;
    `);
  };
  const hidden = new SkinnedMesh(fighter.geometry, material);
  hidden.bind(fighter.skeleton, fighter.bindMatrix);
  hidden.frustumCulled = false;
  // The arena supplies the depth mask. Drawing ordinary suits last keeps visible
  // limbs solid even when two fighters overlap or one arm crosses its own chest.
  hidden.renderOrder = 1;
  fighter.renderOrder = 2;
  fighter.add(hidden);
  return material;
}
