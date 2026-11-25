import { GroupCamera } from './GroupCamera';
import { SkylineView, type SkylineMap, type MatchView } from './SkylineView';
import * as THREE from 'three';
import type { RigDefinition } from '../game/rigDefinition';
import { PracticeTargetView } from './PracticeTargetView';
import { RagdollView } from './RagdollView';
import { fixtureTerrain } from './fixtureTerrain';
import { JointOverlay } from './JointOverlay';

interface SceneOptions {
  focusHeight?: number;
  followRig?: boolean;
  terrainBoxes?: Float32Array;
  target?: Float32Array;
  map?: SkylineMap;
}

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
  private readonly resizeObserver?: ResizeObserver;
  private opponent?: RagdollView;
  private skyline?: SkylineView;
  private groupCamera?: GroupCamera;
  private readonly practice?: PracticeTargetView;
  private readonly rig?: RagdollView;
  private readonly joints?: JointOverlay;

  private readonly followRig: boolean;
  constructor(private readonly container: HTMLElement, definition?: RigDefinition, options: SceneOptions = {}) {
    const { focusHeight = 1.7, terrainBoxes, target, map } = options;
    this.followRig = options.followRig ?? false;
    try {
      this.renderer.setPixelRatio(Math.min(window.devicePixelRatio, 1.5));
      this.renderer.shadowMap.enabled = true;
      this.renderer.shadowMap.type = THREE.PCFSoftShadowMap;
      this.renderer.setClearColor('#182c38');
      this.renderer.domElement.setAttribute('aria-label', 'Live physics fixture on a floating platform');
      container.append(this.renderer.domElement);
      this.camera.position.set(14, 13, 16);
      this.camera.lookAt(0, 0.8, 0);
      if (definition) {
        this.rig = new RagdollView(definition);
        this.joints = new JointOverlay(definition.joints.length);
        this.scene.add(this.rig.root, this.joints.root);
        this.cube.visible = false;
        this.camera.position.set(4, 3.8, 6);
        this.camera.lookAt(0, focusHeight, 0);
        this.renderer.domElement.setAttribute('aria-label', terrainBoxes && terrainBoxes.length > 6 ? 'Live physical ragdoll on low stairs' : 'Live fifteen-body physical ragdoll on a platform');
      }

      if (target) {
        this.practice = new PracticeTargetView(target);
        this.scene.add(this.practice.root);
        this.renderer.domElement.setAttribute('aria-label', 'Live ragdoll and suspended target in strike practice');
      }
      this.cube.castShadow = true;
      this.scene.add(this.cube);
      if (map) {
        this.skyline = new SkylineView(map);
        this.groupCamera = new GroupCamera();
        this.scene.add(this.skyline.root);
        this.renderer.setClearColor(0xb8cfce);
        this.scene.fog = new THREE.FogExp2(0xb8cfce,0.013);
      } else this.scene.add(fixtureTerrain(terrainBoxes));
      this.colliders.frustumCulled = false;
      this.colliders.renderOrder = 1;
      this.scene.add(this.colliders);
      if (!map) {
        const grid = new THREE.GridHelper(12, 12, '#b0d1c5', '#8aada5');
        grid.position.y = 0.005;
        this.scene.add(grid);
      }
      this.scene.add(new THREE.HemisphereLight('#e1f4ff', '#38463b', 2.3));
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
    } catch (error) {
      this.dispose();
      throw error;
    }
  }

  addOpponent(definition: RigDefinition): void {
    this.opponent = new RagdollView(definition, '#559ed8');
    this.scene.add(this.opponent.root);
    this.renderer.domElement.setAttribute('aria-label', this.skyline ? 'Two fighters on the Skyline rooftop' : 'Two fighters on the local sparring platform');
  }

  renderSkyline(first:Float32Array, second:Float32Array, props:Float32Array, match:MatchView, dt:number):void {
    this.skyline!.update(props,match.hazard);
    this.rig!.root.visible=match.visible[0];
    this.opponent!.root.visible=match.visible[1];
    this.opponent!.update(second,-1);
    const present=[first,second].filter((_,id)=>match.visible[id]);
    this.groupCamera!.update(this.camera,present,match.phase==='countdown' || match.phase==='match_results',dt);
    this.render(first,undefined,undefined,-1);
  }

  renderDuel(first: Float32Array, second: Float32Array): void {
    this.opponent!.update(second, -1);
    const x = (first[28]! + second[28]!) / 2;
    const y = Math.max(0.9, (first[29]! + second[29]!) / 2);
    const z = (first[30]! + second[30]!) / 2;
    const spread = Math.hypot(first[28]! - second[28]!, first[29]! - second[29]!, first[30]! - second[30]!);
    const distance = 2.4 + spread / Math.min(1, this.camera.aspect);
    this.camera.position.set(x + distance, y + distance * 0.45, z + distance * 0.48);
    this.camera.lookAt(x, y, z);
    this.render(first, undefined, undefined, -1);
  }

  render(state: Float32Array, lines?: Float32Array, jointStates?: Float32Array, selectedBody = 0, poseTargets?: Float32Array, target?: Float32Array): void {
    if (this.rig) this.rig.update(state, selectedBody);
    else {
      this.cube.position.set(state[0]!, state[1]!, state[2]!);
      this.cube.quaternion.set(state[3]!, state[4]!, state[5]!, state[6]!);
    }
    if (target) this.practice?.update(target);
    this.joints?.update(jointStates, poseTargets);
    if (this.practice) {
      this.camera.position.set(state[28]! + 3.0, 2.15, state[30]! + 1.7);
      this.camera.lookAt(state[28]!, 1.1, state[30]! - 0.22);
    } else if (this.followRig) {
      this.camera.position.set(state[28]! + 4, 3.8, state[30]! + 6);
      this.camera.lookAt(state[28]!, 0.85, state[30]!);
    }
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
    this.resizeObserver?.disconnect();
    this.scene.traverse(object => {
      if (object instanceof THREE.Mesh || object instanceof THREE.LineSegments || object instanceof THREE.Points) {
        object.geometry.dispose();
        const materials = Array.isArray(object.material) ? object.material : [object.material];
        for (const material of materials) material.dispose();
      }
      if (object instanceof THREE.DirectionalLight) object.shadow.dispose();
    });
    this.renderer.dispose();
    // Scene changes construct another renderer. Release this context explicitly
    // so repeated inspection cannot exhaust the browser's active-context limit.
    this.renderer.forceContextLoss();
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
