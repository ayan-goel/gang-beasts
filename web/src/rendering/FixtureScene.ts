import { TourView } from './tour/TourView';
import { stagePalette } from './tour/palette';
import stationSkyUrl from './art/station-sky.webp';
import deckPrintUrl from './art/studio-deck.webp';
import { recentBreakawayMount } from '../audio/breakawaySound';
import type { SkylineContent } from '../lobby/content';
import { createSceneRenderer } from './createSceneRenderer';
import { GraphicsRecovery } from './GraphicsRecovery';
import type { LandingEvent } from '../audio/landingSound';
import { graphicsDriver, skylineQuality } from './skylineQuality';
import { applySkylineQuality } from './applySkylineQuality';
import { graphicsPreference, subscribeGraphics } from './graphicsPreference';
import { FighterSkin } from './fighter/FighterSkin';
import { ContactNormals } from './ContactNormals';
import { observeCameraMotion } from './cameraMotion';
import { GroupCamera } from './GroupCamera';
import { SkylineView, type SkylineMap, type MatchView } from './SkylineView';
import * as THREE from 'three';
import type { RigDefinition } from '../game/rigDefinition';
import { PracticeTargetView } from './PracticeTargetView';
import { RagdollView } from './RagdollView';
import { fixtureTerrain } from './fixtureTerrain';
import { JointOverlay } from './JointOverlay';

interface SceneOptions {
  color?: string;
  focusHeight?: number;
  followRig?: boolean;
  terrainBoxes?: Float32Array;
  target?: Float32Array;
  map?: SkylineMap;
  model?: THREE.Object3D;
  kit?: number;
  online?: boolean;
}

export class FixtureScene {
  private readonly renderer = createSceneRenderer();
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
  private readonly recovery?: GraphicsRecovery;
  onGraphicsChange?: (lost: boolean) => void;
  get graphicsInterrupted() { return this.recovery?.interrupted ?? false; }

  private readonly resizeObserver?: ResizeObserver;
  private readonly unsubscribeMotion?: () => void;
  private readonly unsubscribeGraphics?: () => void;
  private readonly opponents: (RagdollView | FighterSkin)[] = [];
  private skyline?: SkylineView | TourView;
  private arenaId?: string;
  private arenaName = 'Skyline rooftop';
  private deckPrint?: THREE.Texture;
  private stationSky?: THREE.Texture;
  readonly groupCamera?: GroupCamera;
  private readonly practice?: PracticeTargetView;
  private readonly rig?: RagdollView | FighterSkin;
  private readonly model?: THREE.Object3D;
  readonly contactNormals?: ContactNormals;
  private readonly joints?: JointOverlay;

  private readonly followRig: boolean;
  constructor(private container: HTMLElement, definition?: RigDefinition, options: SceneOptions = {}) {
    const { focusHeight = 1.7, terrainBoxes, target, map } = options;
    this.model = options.model;
    this.followRig = options.followRig ?? false;
    try {
      const driver = map ? graphicsDriver(this.renderer.getContext()) : '';
      this.renderer.shadowMap.enabled = true;
      this.renderer.setClearColor('#182c38');
      this.renderer.domElement.setAttribute('aria-label', 'Live physics fixture on a floating platform');
      container.append(this.renderer.domElement);
      this.camera.position.set(14, 13, 16);
      this.camera.lookAt(0, 0.8, 0);
      if (definition) {
        if (!map) {
          this.contactNormals = new ContactNormals();
          this.scene.add(this.contactNormals);
        }
        if (this.model) this.rig = new FighterSkin(this.model, definition, options.color ?? '#ff5047', options.kit ?? 0);
        else {
          this.rig = new RagdollView(definition, options.color);
          this.joints = new JointOverlay(definition.joints.length);
          this.scene.add(this.joints.root);
        }
        this.scene.add(this.rig.root);
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
        if (!definition) throw new Error('Arena framing requires fighter geometry');
        this.groupCamera = new GroupCamera(definition.bodies, this.rig instanceof FighterSkin ? this.rig.radii : undefined);
        this.setArena(map);
        this.unsubscribeMotion = observeCameraMotion(reduced => { this.groupCamera!.reducedMotion = reduced; }, options.online);
      } else this.scene.add(fixtureTerrain(terrainBoxes));
      this.colliders.frustumCulled = false;
      this.colliders.renderOrder = 1;
      this.scene.add(this.colliders);
      if (!map) {
        const grid = new THREE.GridHelper(12, 12, '#b0d1c5', '#8aada5');
        grid.position.y = 0.005;
        this.scene.add(grid);
      }
      this.scene.add(new THREE.HemisphereLight(map ? '#e7f8ff' : '#e1f4ff',map ? '#b38c96' : '#38463b',map ? 1.8 : 2.3));
      const sun = new THREE.DirectionalLight(map ? '#ffebc9' : '#fff0d9',map ? 2.7 : 3);
      sun.position.set(-6, map ? 8 : 12, 6);
      sun.castShadow = true;
      sun.shadow.camera.left = -9;
      sun.shadow.camera.right = 9;
      sun.shadow.camera.top = 9;
      sun.shadow.camera.bottom = -9;
      sun.shadow.normalBias = 0.03;
      this.scene.add(sun);
      const applyQuality = () => applySkylineQuality(this.renderer, this.scene, sun,
        skylineQuality(driver, window.devicePixelRatio, options.online ? 'high' : map ? graphicsPreference() : 'medium'));
      const refreshSize = () => {
        if (this.graphicsInterrupted) return;
        applyQuality();
        this.resize();
      };
      this.recovery = new GraphicsRecovery(this.renderer.domElement, !!options.online, lost => {
        if (!lost) refreshSize();
        this.onGraphicsChange?.(lost);
      });
      if (map && !options.online) this.unsubscribeGraphics = subscribeGraphics(refreshSize);
      this.resizeObserver = new ResizeObserver(refreshSize);
      this.resizeObserver.observe(container);
      refreshSize();
    } catch (error) {
      this.dispose();
      throw error;
    }
  }

  setArena(map: SkylineMap) {
    const id = map.id ?? 'skyline';
    if (this.arenaId === id) return;
    const loader = new THREE.TextureLoader();
    const isTour = !!map.tour;
    const haze = isTour ? stagePalette(id).haze : '#ffc6a3';
    const floor = loader.load(isTour ? new URL(`./tour/art/${id}-floor.webp`,import.meta.url).href : deckPrintUrl);
    floor.colorSpace = THREE.SRGBColorSpace;
    floor.anisotropy = Math.min(4,this.renderer.capabilities.getMaxAnisotropy());
    if (isTour) floor.wrapS = floor.wrapT = THREE.RepeatWrapping;
    let next: SkylineView | TourView;
    try {
      next = isTour ? new TourView(map,floor) : new SkylineView(map,floor);
    } catch (error) { floor.dispose(); throw error; }
    if (this.skyline) {
      this.skyline.root.removeFromParent();
      disposeMeshes(this.skyline.root);
    }
    this.skyline = next;
    this.scene.add(next.root);
    this.arenaId = id;
    this.arenaName = map.tour?.name ?? 'Skyline rooftop';
    this.groupCamera?.setArena(next.obstacles,map);
    this.deckPrint?.dispose();
    this.deckPrint = floor;
    this.scene.background = new THREE.Color(haze);
    this.stationSky?.dispose();
    this.stationSky = loader.load(isTour ? new URL(`./tour/art/${id}-sky.webp`,import.meta.url).href : stationSkyUrl, sky => {
      // A slow image from the last round must never replace the current arena's sky.
      if (this.stationSky === sky) this.scene.background = sky;
    });
    this.stationSky.colorSpace = THREE.SRGBColorSpace;
    this.stationSky.mapping = THREE.EquirectangularReflectionMapping;
    this.renderer.setClearColor(haze);
    this.scene.fog = new THREE.FogExp2(haze,isTour ? .017 : .015);
    this.updateArenaLabel();
  }

  private updateArenaLabel() {
    const count = this.opponents.length + 1;
    const arena = this.skyline ? this.arenaName : 'local sparring platform';
    this.renderer.domElement.setAttribute('aria-label',(count === 2 ? 'Two' : count) + ' fighters on the ' + arena);
  }

  async prepareSkyline(signal: AbortSignal) {
    // A detached lobby canvas has no layout size. A small real draw also loads the shadow and skin programs.
    this.renderer.setSize(16, 16, false);
    this.renderer.render(this.scene, this.camera);
    const gl = this.renderer.getContext() as WebGL2RenderingContext;
    const fence = gl.fenceSync(gl.SYNC_GPU_COMMANDS_COMPLETE, 0);
    if (!fence) throw new Error('Could not prepare Skyline graphics');
    gl.flush();
    try {
      while (!gl.isContextLost()) {
        signal.throwIfAborted();
        const status = gl.clientWaitSync(fence, 0, 0);
        if (status === gl.ALREADY_SIGNALED || status === gl.CONDITION_SATISFIED) return;
        if (status === gl.WAIT_FAILED) break;
        await new Promise(resolve => setTimeout(resolve, 16));
      }
      throw new Error('Skyline graphics preparation was interrupted');
    } finally { gl.deleteSync(fence); }
  }

  mountSkyline(container: HTMLElement, team: string, kit: number) {
    if (this.rig instanceof FighterSkin) this.rig.setKit(team, kit);
    this.resizeObserver?.unobserve(this.container);
    this.container = container;
    container.append(this.renderer.domElement);
    this.resizeObserver?.observe(container);
    this.resize();
  }

  addOpponent(definition: RigDefinition, color = '#24aeeb', kit = 1): void {
    const opponent = this.model ? new FighterSkin(this.model, definition, color, kit) : new RagdollView(definition, color);
    this.opponents.push(opponent);
    this.scene.add(opponent.root);
    this.updateArenaLabel();
  }

  private playerPoses: Float32Array[] = [];
  private readonly soundPoint = new THREE.Vector3();

  breakawaySource(map: SkylineContent['map'], tick: number) {
    if (map.tour) return;
    const mount = recentBreakawayMount(map, tick);
    return mount && this.audioPosition(this.soundPoint.fromArray(mount));
  }

  private audioPosition(point: THREE.Vector3) {
    const distance = point.distanceTo(this.camera.position);
    return { distance, pan: THREE.MathUtils.clamp(point.project(this.camera).x, -1, 1) };
  }

  impactSources(fighters: readonly {health: number; footstep?: number | null; landing?: LandingEvent | null}[], arenaTick: number) {
    return this.playerPoses.map((pose, slot) => {
      const pelvis = this.soundPoint.fromArray(pose, 28);
      const { distance, pan } = this.audioPosition(pelvis);
      const {health, landing, footstep} = fighters[slot]!;
      return {health, distance, pan, footstep: footstep && arenaTick - footstep <= 12 ? footstep : undefined,
        landing: landing && arenaTick - landing.tick <= 12 ? landing : undefined};
    });
  }

  renderSkyline(players: Float32Array[], props: Float32Array, match: MatchView, dt: number, owner?: Float32Array, follow?: number | null): boolean {
    this.playerPoses = players;
    this.skyline!.update(props, match.hazard, dt, players, match);
    this.rig!.root.visible = match.visible[0]!;
    this.opponents.forEach((rig, i) => {
      rig.root.visible = match.visible[i + 1]!;
      if (rig.root.visible) rig.update(players[i + 1]!, -1);
    });
    const present = follow != null && match.visible[follow] ? [players[follow]!] : players.filter((_, slot) => match.visible[slot]);
    const turned = this.groupCamera!.update(this.camera, present, match.phase === 'countdown' || match.phase === 'match_results', dt, owner);
    this.render(players[0]!, undefined, undefined, -1);
    return turned;
  }

  renderDuel(first: Float32Array, second: Float32Array): void {
    this.opponents[0]!.update(second, -1);
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
    if (!this.graphicsInterrupted) this.renderer.render(this.scene, this.camera);
  }

  dispose(): void {
    this.recovery?.dispose();
    this.resizeObserver?.disconnect();
    this.unsubscribeGraphics?.();
    this.unsubscribeMotion?.();
    for (const fighter of [this.rig, ...this.opponents]) {
      if (fighter instanceof FighterSkin) {
        // Geometry is shared by the cached GLB and every live instance.
        fighter.root.removeFromParent();
        fighter.dispose();
      }
    }
    disposeMeshes(this.scene);
    this.scene.traverse(object => {
      if (object instanceof THREE.DirectionalLight) object.shadow.dispose();
    });
    this.deckPrint?.dispose();
    this.stationSky?.dispose();
    this.stationSky = undefined;
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


function disposeMeshes(root: THREE.Object3D) {
  root.traverse(object => {
    if (object instanceof THREE.Mesh || object instanceof THREE.LineSegments || object instanceof THREE.Points) {
      object.geometry.dispose();
      for (const material of Array.isArray(object.material) ? object.material : [object.material]) material.dispose();
    }
  });
}
