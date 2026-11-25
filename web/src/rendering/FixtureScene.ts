import * as THREE from 'three';

export class FixtureScene {
  private readonly renderer = new THREE.WebGLRenderer({ antialias: true });
  private readonly scene = new THREE.Scene();
  private readonly camera = new THREE.PerspectiveCamera(45, 1, 0.1, 150);
  private readonly colliders = new THREE.LineSegments(
    new THREE.BufferGeometry(),
    new THREE.LineBasicMaterial({ color: '#ffe1a6', depthTest: false }),
  );
  private readonly cube = new THREE.Mesh(
    new THREE.BoxGeometry(1, 1, 1),
    new THREE.MeshStandardMaterial({ color: '#f58465', roughness: 0.65 }),
  );
  private readonly resizeObserver: ResizeObserver;

  constructor(private readonly container: HTMLElement) {
    this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 1.5));
    this.renderer.shadowMap.enabled = true;
    this.renderer.shadowMap.type = THREE.PCFSoftShadowMap;
    this.renderer.setClearColor('#182c38');
    this.renderer.domElement.setAttribute('aria-label', 'Live physics fixture on a floating platform');
    container.append(this.renderer.domElement);
    this.camera.position.set(14, 13, 16);
    this.camera.lookAt(0, 0.8, 0);

    const platform = new THREE.Mesh(
      new THREE.BoxGeometry(12, 1, 12),
      new THREE.MeshStandardMaterial({ color: '#789a96', roughness: 0.95 }),
    );
    platform.position.y = -0.5;
    platform.receiveShadow = true;
    this.cube.castShadow = true;
    this.scene.add(platform, this.cube);
    this.colliders.frustumCulled = false;
    this.colliders.renderOrder = 1;
    this.scene.add(this.colliders);
    const grid = new THREE.GridHelper(12, 12, '#b0d1c5', '#8aada5');
    grid.position.y = 0.005;
    this.scene.add(grid, new THREE.HemisphereLight('#e1f4ff', '#38463b', 2.3));
    const sun = new THREE.DirectionalLight('#fff0d9', 3);
    sun.position.set(-5, 12, 6);
    sun.castShadow = true;
    sun.shadow.mapSize.set(1024, 1024);
    sun.shadow.camera.left = -9;
    sun.shadow.camera.right = 9;
    sun.shadow.camera.top = 9;
    sun.shadow.camera.bottom = -9;
    sun.shadow.normalBias = 0.03;
    this.scene.add(sun);
    this.resizeObserver = new ResizeObserver(() => this.resize());
    this.resizeObserver.observe(container);
    this.resize();
  }

  render(state: Float32Array, lines?: Float32Array): void {
    this.cube.position.set(state[0]!, state[1]!, state[2]!);
    this.cube.quaternion.set(state[3]!, state[4]!, state[5]!, state[6]!);
    this.colliders.visible = !!lines?.length;
    if (lines?.length) {
      const attribute = this.colliders.geometry.getAttribute('position');
      if (!attribute || attribute.array.length !== lines.length) {
        this.colliders.geometry.setAttribute('position', new THREE.BufferAttribute(lines, 3));
      } else {
        attribute.array.set(lines);
        attribute.needsUpdate = true;
      }
    }
    this.renderer.render(this.scene, this.camera);
  }

  dispose(): void {
    this.resizeObserver.disconnect();
    this.scene.traverse(object => {
      if (object instanceof THREE.Mesh || object instanceof THREE.LineSegments) {
        object.geometry.dispose();
        const materials = Array.isArray(object.material) ? object.material : [object.material];
        for (const material of materials) material.dispose();
      }
      if (object instanceof THREE.DirectionalLight) object.shadow.dispose();
    });
    this.renderer.dispose();
    this.renderer.domElement.remove();
  }

  private resize(): void {
    const width = Math.max(1, this.container.clientWidth);
    const height = Math.max(1, this.container.clientHeight);
    this.camera.aspect = width / height;
    this.camera.updateProjectionMatrix();
    this.renderer.setSize(width, height);
  }
}
