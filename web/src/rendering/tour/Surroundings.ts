import { Color, Group } from 'three';
import { SkylineDetails } from '../SkylineDetails';
import type { ArenaPoint } from '../SkylineView';
import type { StagePalette } from './palette';

export interface Mobile { group: Group; speed: number; axis: 'x' | 'y' | 'z' }

export function dressSurroundings(root: Group, p: StagePalette): Mobile[] {
  const a = new SkylineDetails();
  const mobiles: Mobile[] = [];
  const ink = new Color(p.ink).getHex(), edge = new Color(p.edge).getHex(), accent = new Color(p.accent).getHex();
  const cream = 0xffe6bb, rust = 0xac675c, steel = 0x687c89;
  function mobile(pos: ArenaPoint, speed: number, axis: Mobile['axis'], build: (art: SkylineDetails)=>void) {
    const group = new Group();
    group.position.set(...pos);
    const art = new SkylineDetails();
    build(art);
    art.attach(group); root.add(group); mobiles.push({group,speed,axis});
  }
  function gantry(x: number,z: number,w: number,top: number,bottom = -13) {
    for (const side of [-1,1]) {
      const x0=x+side*w/2;
      for (const dx of [-.48,.48]) a.rod([x0+dx,bottom,z],[x0+dx,top,z],.12,edge);
      for (let y=bottom; y<top-1; y+=1.8) {
        a.rod([x0-.48,y,z],[x0+.48,Math.min(y+1.8,top),z],.07,accent);
        a.rod([x0+.48,y,z],[x0-.48,Math.min(y+1.8,top),z],.07,accent);
      }
    }
    a.add([w+1,.46,.85],[x,top,z],edge);
    a.add([w+1,.13,.9],[x,top+.28,z],accent);
  }
  function tree(x:number,y:number,z:number,scale:number, palm=false) {
    a.rod([x,y,z],[x+.24*scale,y+2.9*scale,z],.17*scale,0x846550);
    if (palm) {
      for (let i=0;i<7;i++) {
        const t=i*.9, cx=x+.24*scale, cy=y+2.9*scale;
        const tip: ArenaPoint=[cx+2*scale*Math.cos(t),cy-.50*scale,z+2*scale*Math.sin(t)];
        a.rod([cx,cy,z],tip,.18*scale, i%2 ? 0x668b65 : 0x819c62);
      }
    } else {
      a.oval([1.23*scale,1.42*scale,1.12*scale],[x,y+2.9*scale,z],0x64815d);
      for (const side of [-1,1]) a.oval([.81*scale,.93*scale,.85*scale],[x+side*.77*scale,y+2.45*scale,z],0x79935a);
    }
  }
  function mountain(x:number,z:number,height:number,snow:boolean) {
    const y=-4;
    a.drum(.22,height*.74,height,[x,y+height/2,z],edge,7);
    if (snow) a.drum(.10,height*.27,height*.38,[x,y+height*.84,z],0xdfe7e1,7);
  }
  function windmill(x:number,z:number,h:number) {
    a.drum(.25,.8,h,[x,h/2-3,z],cream);
    mobile([x,h-3,z+.15],.36,'z',art=>{
      for (let i=0;i<3;i++) {
        const angle=i*2*Math.PI/3;
        art.rod([0,0,0],[4*Math.sin(angle),4*Math.cos(angle),.03],.18,cream);
      }
      art.oval([.45,.45,.4],[0,0,0],accent);
    });
  }
  function wheel(x:number,y:number,z:number,r:number,cabins:boolean) {
    for (const side of [-1,1]) a.rod([x+side*r*.76,-5,z],[x,y,z],.23,edge);
    mobile([x,y,z],cabins ? .065 : .28,'z',art=>{
      art.hoop(r,.18,[0,0,0],accent);
      art.hoop(r*.87,.08,[0,0,0],cream);
      for (let i=0;i<12;i++) {
        const angle=i*Math.PI/6, dx=Math.cos(angle)*r, dy=Math.sin(angle)*r;
        art.rod([0,0,0],[dx,dy,0],.09,edge);
        if (cabins) art.oval([.6,.42,.7],[dx,dy,0],i%2 ? cream : 0xed847c);
        else art.add([.56,.82,.98],[dx,dy,0],rust);
      }
      art.oval([.7,.7,.7],[0,0,0],cream);
    });
  }
  function stack(x:number,z:number,height:number) {
    a.drum(.78,1.25,height,[x,height/2-8,z],ink,12);
    for(let ring=0;ring<3;ring++) a.drum(.96,1,.54,[x,height-9-ring*1.4,z],accent,12);
  }
  function pylon(x:number,z:number,h:number) {
    a.rod([x,-13,z],[x,h,z],.21,steel);
    a.add([3.4,.3,.42],[x,h,z],edge);
    for(const side of [-1,1]) a.drum(.23,.23,.45,[x+side*1.35,h+.26,z],accent);
  }
  function ground(top:number,paint:number) {
    a.add([110,.6,110],[0,top-.3,0],paint);
  }
  switch (p.land) {
    case 'atoll':
      for(const [x,z,s] of [[-14,-12,1.15],[21,-19,1.4],[18,13,.9]] as const) {
        a.drum(2.9*s,4.1*s,1.15,[x,-1,z],0xcdab72,9);
        tree(x,-.5,z,s,true); tree(x+1.8,-.5,z-.9,s*.7,true);
      }
      mountain(-3,-36,13,false);
      a.drum(1.2,2.2,.78,[-3,8.15,-36],0xdc865d,7);
      for (let i=0;i<6;i++) {
        const size=.7+i*.23;
        a.oval([size,size*.76,size*.82],[-3+i*.62,9.1+i*.68,-36],i<2 ? 0xc09b8f : 0xc5aba8);
      }
      for(let i=0;i<7;i++) {
        const angle=i*1.88, r=11+i%3*2.6;
        a.drum(.23+i%3*.13,.86+i%2*.4,.9,[Math.cos(angle)*r,-.67,Math.sin(angle)*r],0x78847a,6);
      }
      break;
    case 'flight':
      for(let i=0;i<23;i++) {
        const angle=i*2.39,r=19+i%5*5;
        const x=Math.sin(angle)*r,z=Math.cos(angle)*r,y=-6.8-i%3*2;
        a.oval([3.5,1.5,2.4],[x,y,z],0xffebd3);
        a.oval([2.8,1.2,2.2],[x+2,y-.3,z],0xf2d6cd);
      }
      break;
    case 'polar':
    case 'alpine':
      for(const [x,z,h] of [[-18,-21,11],[9,-28,18],[24,12,12],[-27,11,10]] as const) mountain(x,z,h,true);
      if(p.land==='alpine') {
        pylon(-12,-14,5); pylon(16,-20,8);
        a.rod([-13.35,5.4,-14],[14.65,8.4,-20],.045,ink);
        a.rod([-10.65,5.4,-14],[17.35,8.4,-20],.045,ink);
      } else {
        gantry(-10,-15,2.1,5,-2);
        a.rod([-10,5,-15],[-10,8,-15],.085,steel);
        for(let i=0;i<4;i++) a.add([2.3-i*.44,.12,.16],[-10,5.7+i*.53,-15],cream);
      }
      break;
    case 'harbor':
      a.drum(1.9,3.9,2.8,[-13,-1,-17],0x71807c,9);
      a.drum(.84,1.3,5,[-13,2.6,-17],cream);
      a.drum(1,1.13,1.1,[-13,2.6,-17],rust);
      a.drum(1.12,1.12,1,[-13,5.6,-17],accent,8);
      a.drum(.06,1.48,.91,[-13,6.55,-17],ink,8);
      break;
    case 'port':
    case 'works':
      for(const [x,z,h] of [[-16,-19,8],[15,-23,12]] as const) {
        gantry(x,z,5.2,h); a.rod([x-2.6,h,z],[x+8.9,h+4,z],.25,accent);
        a.rod([x+8.8,h+4,z],[x+8.8,1,z],.055,ink);
        for(let k=0;k<6;k++) a.add([3.9,1.5,1.9],[x+(k%3-1)*4,-3+Math.floor(k/3)*1.51,z+3],k%2?rust:steel);
      }
      break;
    case 'windfarm':
      for(const [x,z,h] of [[-14,-17,9],[13,-20,12],[-25,8,11],[25,9,15]] as const) windmill(x,z,h);
      break;
    case 'towers':
      for(const [x,z,height] of [[-18,-16,23],[11,-25,26],[25,5,20],[-26,7,19]] as const) {
        a.add([7,height,7],[x,height/2-18,z],edge);
        for(let y=-13; y<height-18; y+=2) for(let i=-2;i<=2;i+=1.5) {
          a.add([.96,.85,.03],[x+i,y,z+3.51],(i+y)%2 ? 0x5e6d80 : accent);
          a.add([.03,.85,.96],[x-3.51,y,z+i],0x637486);
        }
      }
      break;
    case 'metro':
      ground(-5,0x49505e);
      for(const x of [-13,13]) {
        a.add([2.4,5.6,35],[x,-1.6,0],edge);
        for(let z=-14;z<15;z+=4) {
          a.add([.42,5.9,.37],[x - Math.sign(x)*1.4,-1.65,z],ink);
          a.add([.26,.20,2.8],[x - Math.sign(x)*1.34,.83,z],cream);
        }
      }
      gantry(0,-17,8,3.3,-5);
      break;
    case 'kiln':
    case 'steel':
      ground(-7.3,p.land==='kiln'?0xbc7152:0x785b5d);
      for(let i=-4;i<5;i++) {
        a.add([2.0,.08,52],[i*5,-7.22,-2],0x8a625a);
        a.add([.14,.09,52],[i*5+.82,-7.20,-2],0xffa459);
      }
      for(const [x,z,h] of [[-15,-18,14],[-11,-23,18],[18,-20,16]] as const) stack(x,z,h);
      gantry(2,-15,12,4);
      if(p.land==='steel') {
        for(const x of [-13,14]) a.add([4.5,2.5,6],[x,-4,-9],ink);
      } else {
        for(const x of [-11,12]) a.drum(2.3,3.3,4.8,[x,-3.9,-11],rust,10);
      }
      break;
    case 'parade':
    case 'circus':
    case 'pier': {
      if(p.land!=='pier') {
        ground(-4.8,0xb4867e);
        if(p.land==='parade') {
          a.add([14,.08,90],[0,-4.74,0],0x876f7d);
          for(let i=-14;i<15;i++) a.add([.11,.025,.96],[0,-4.68,i*2],cream);
          for(const x of [-8.2,8.2]) a.add([2.4,.14,90],[x,-4.65,0],0xd99c8a);
        } else {
          for(let i=0;i<4;i++) a.hoop(5+i*.6,.08,[0,-4.46,0],i%2?edge:cream,true);
        }
      }
      if(p.land==='pier') wheel(2,4,-18,8,true);
      for(const [x,z] of [[-14,-15],[16,-18],[23,8]] as const) {
        a.drum(2.8,2.8,2.6,[x,-2,z],edge,10);
        a.drum(.09,3.4,2.6,[x,.55,z],accent,10);
        a.rod([x,1.7,z],[x,3.1,z],.07,ink); a.fin(.7,.6,.12,[x+.3,2.9,z],rust);
      }
      for(const side of [-1,1]) {
        const x=side*13;
        a.rod([x,-5,-12],[x,6,-12],.14,ink);
      }
      for(let i=0;i<14;i++) {
        const x=-13+i*2, y=4.2+1.8*(x*x/169);
        if(i<13) a.rod([x,y,-12],[x+2,4.2+1.8*((x+2)**2/169),-12],.03,ink);
        a.fin(.57,-.79,.08,[x,y-.39,-12],i%2?cream:rust);
      }
      break;
    }
    case 'space':
      for(let i=0;i<14;i++) {
        const t=i*2.4, r=18+i%4*4;
        a.drum(.35+i%4*.2,.8+i%3*.24,1+i%4*.3,[Math.cos(t)*r,-4+i%7,Math.sin(t)*r],steel,5);
      }
      a.oval([6,6,6],[-16,4,-35],0xc59a9d);
      a.hoop(8.7,.35,[-16,4,-35],0xe5c79b,true);
      a.hoop(9.4,.15,[-16,4,-35],0xb9a9c2,true);
      a.drum(1.2,1.6,3.4,[-12,2,-18],cream);
      for(const sign of [-1,1]) {
        a.add([4.5,.16,2.8],[-12+sign*4,2,-18],0x677bb0);
        for(let i=-1;i<=1;i++) a.add([.075,.2,2.8],[-12+sign*4+i,2,-18],accent);
      }
      break;
    case 'reservoir':
      for(const side of [-1,1]) {
        a.add([13,6,2.5],[side*12,-2.4,-17],edge);
        a.add([13,.4,2.7],[side*12,.8,-17],cream);
        mountain(side*22,-26,11,false);
      }
      for(let i=-2;i<=2;i++) {
        a.add([.55,6,2.7],[i*1.95,-2.5,-17],ink);
        a.add([1.45,4.8,.2],[i*1.95+.96,-2.8,-15.5],0xabcac1);
      }
      break;
    case 'railway':
      ground(-4.6,0x9aa781);
      for(let z=-45;z<=45;z+=1.8) a.add([4,.23,.37],[0,-3.9,z],0x7e6254);
      for(const x of [-1.2,1.2]) a.add([.13,.17,93],[x,-3.7,0],steel);
      for(let i=0;i<7;i++) pylon(i%2?-13:14,-25+i*8,3.5+i%2);
      break;
    case 'observatory':
      for(const [x,z] of [[-13,-18],[17,-23],[24,17]] as const) {
        a.drum(1,1.5,4,[x,-2,z],edge);
        a.drum(3.2,.6,1.2,[x,.6,z],cream,28);
        for(const side of [-1,1]) a.rod([x+side*2.7,1.1,z],[x,3,z],.09,steel);
        a.oval([.36,.36,.36],[x,3,z],accent);
      }
      break;
    case 'orchard':
      wheel(-12,1,-14,4,false);
      for(let i=0;i<18;i++) {
        const x=(i%9-4)*4.3, z=-20-Math.floor(i/9)*4;
        a.drum(3.4,3.8,2,[x,-1.3,z],0x87a175,9);
        tree(x,-.3,z,1+i%3*.12);
      }
      a.add([5.2,3.6,4.1],[12,1,-15],rust);
      a.drum(.12,3.8,2.4,[12,3.9,-15],ink,4);
      break;
    case 'concert':
      ground(-5.2,ink);
      gantry(0,-13,18,6.4,-5);
      for(let i=-3;i<=3;i++) {
        a.drum(.37,.61,.7,[i*2.5,5.8,-13],ink);
        a.drum(.60,.60,.09,[i*2.5,5.41,-13],i%2?accent:0xfb8f9a);
      }
      for(const side of [-1,1]) for(let i=0;i<3;i++) {
        a.add([2.1,2.1,1.7],[side*12,i*2.14-2,-9],ink);
        a.hoop(.63,.11,[side*12,i*2.14-2,-8.12],steel);
        a.oval([.27,.27,.13],[side*12,i*2.14-2,-8.10],edge);
      }
      break;
  }
  const scenery = new Group();
  a.attach(scenery);
  // Distant decoration is outside the playable footprint and does not darken the fighters' shadow map.
  for (const mesh of scenery.children) mesh.castShadow = false;
  root.add(scenery);
  return mobiles;
}
