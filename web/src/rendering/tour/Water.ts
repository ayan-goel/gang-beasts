import type { ArenaObject } from '../SkylineView';
import { Color, DoubleSide, Mesh, PlaneGeometry, ShaderMaterial, Vector2, Vector4 } from 'three';

const vertex = `
  uniform float time;
  varying vec2 surface;
  void main() {
    vec3 p = position;
    surface = p.xz;
    p.y += .034*sin(p.x*1.43+time*1.6) + .026*sin(p.z*1.75-p.x*.35+time);
    gl_Position = projectionMatrix * modelViewMatrix * vec4(p,1.);
  }`;
const fragment = `
  uniform float time;
  uniform vec3 deep, sky;
  uniform vec4 swimmers[8];
  uniform vec4 shores[8];
  uniform vec2 bearings[8];
  varying vec2 surface;
  void main() {
    vec2 p = surface;
    vec2 warped = p + vec2(sin(p.y*.63+time*.29), sin(p.x*.7-time*.44))*.60;
    float swell = sin(warped.y*4.3 + warped.x*.59 + sin(warped.x*1.24 + time*.55) + time*1.35);
    float streak = smoothstep(.92,.995,swell)*smoothstep(.26,.82,sin(warped.x*1.78-time*.67));
    float big = .5+.5*cos(warped.x*.24-warped.y*.57+time*.63)*sin(warped.y*.42+time*.19);
    float nearShore = 20.;
    for (int i=0; i<8; i++) {
      vec2 delta = p-shores[i].xy;
      vec2 b = bearings[i];
      vec2 local = vec2(dot(delta,b),dot(delta,vec2(-b.y,b.x)));
      vec2 d = abs(local) - shores[i].zw + .22;
      nearShore = min(nearShore, length(max(d,0.)) + min(max(d.x,d.y),0.) - .22);
    }
    float wash = (1.-smoothstep(.02, .90, nearShore))*(.52+.38*sin(nearShore*11.-time*1.6 + .5*sin(p.y*2.)));
    float wakes = 0.;
    for (int i=0; i<8; i++) {
      float d = length(p-swimmers[i].xy);
      float wave = sin(d*14.-time*5.5+float(i));
      wakes += (1.-smoothstep(.26,1.6,d))*smoothstep(.74,.98,wave)*swimmers[i].z;
    }
    vec3 color = mix(deep,sky,.08*big + .10*streak);
    color = mix(color,vec3(.76,.98,.88),clamp(.16*streak + .58*wakes + .36*wash,0.,.78));
    float haze = smoothstep(23.,82.,length(p));
    color = mix(color,sky,.59*haze);
    gl_FragColor = vec4(color,.72 + .20*haze);
    #include <tonemapping_fragment>
    #include <colorspace_fragment>
  }`;

// Physics owns the water level and buoyancy. Tiny visual ripples preserve a stable, readable catch height.
export class ArenaWater {
  private readonly swimmers = Array.from({length:8}, () => new Vector4());
  private readonly shores = Array.from({length:8}, () => new Vector4(0,0,-1000,-1000));
  private readonly bearings = Array.from({length:8}, () => new Vector2(1,0));
  private readonly shoreObjects: ArenaObject[];
  readonly mesh: Mesh<PlaneGeometry, ShaderMaterial>;
  constructor(readonly height: number, water: string, haze: string, objects: ArenaObject[]) {
    this.shoreObjects = objects.filter(obj => obj.kind === 'deck' || obj.kind === 'panel').slice(0,8);
    const material = new ShaderMaterial({
      vertexShader:vertex, fragmentShader:fragment, transparent:true, depthWrite:false, side:DoubleSide,
      uniforms:{time:{value:0}, deep:{value:new Color(water)}, sky:{value:new Color(haze)}, swimmers:{value:this.swimmers}, shores:{value:this.shores}, bearings:{value:this.bearings}},
    });
    this.mesh = new Mesh(new PlaneGeometry(190,190,90,90).rotateX(-Math.PI/2),material);
    this.mesh.position.y = height;
    this.mesh.renderOrder = 2;
    this.mesh.name = 'water';
  }

  update(time: number, poses: readonly Float32Array[], visible: readonly boolean[], props:Float32Array) {
    this.mesh.material.uniforms.time!.value = time;
    this.shoreObjects.forEach((obj,i) => {
      const at = (obj.id-1)*8;
      const shore = this.shores[i]!;
      if (!props[at+7] || Math.abs(props[at+1]!-this.height)>obj.half[1]+.35) {
        shore.set(0,0,-1000,-1000);
        return;
      }
      shore.set(props[at]!,props[at+2]!,obj.half[0],obj.half[2]);
      const x=props[at+3]!, y=props[at+4]!, z=props[at+5]!, w=props[at+6]!;
      const yaw=Math.atan2(2*(w*y+x*z),1-2*(y*y+z*z));
      this.bearings[i]!.x=Math.cos(yaw);
      this.bearings[i]!.y=-Math.sin(yaw);
    });
    this.swimmers.forEach((contact,i) => {
      const pose = poses[i];
      if (!pose || !visible[i]) { contact.z = 0; return; }
      const immersion = Math.abs(pose[29]! - this.height);
      contact.set(pose[28]!, pose[30]!, Math.max(0,1-immersion/.95), 0);
    });
  }
}
