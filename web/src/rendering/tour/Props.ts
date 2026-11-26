import { Color, type Object3D } from 'three';
import { SkylineDetails } from '../SkylineDetails';
import type { ArenaObject } from '../SkylineView';
import type { StagePalette } from './palette';

const ivory = 0xffe8bb;
const glass = 0x385669;
const steel = 0x677988;

export function dressProp(parent: Object3D, obj: ArenaObject, palette: StagePalette) {
  const a = new SkylineDetails();
  const [x, y, z] = obj.half;
  const top = y + .012;
  const accent = new Color(palette.accent).getHex();
  const edge = new Color(palette.edge).getHex();
  const ink = new Color(palette.ink).getHex();

  function truss() {
    const count = Math.max(2,Math.ceil(z*1.35));
    for (const side of [-1,1]) {
      a.rod([side*x,-y,z],[side*x,-y,-z],.055,ink);
      for(let i=0;i<count;i++) {
        const z0=-z+i*2*z/count, z1=z0+2*z/count;
        a.rod([side*x,-y,z0],[side*x,y,z1],.055,accent);
      }
    }
  }
  function bands() {
    for (const sign of [-1, 1]) {
      a.add([2*x+.018,.065,2*z+.018],[0,sign*y*.65,0],ink);
      a.add([.09,2*y+.018,2*z+.018],[sign*x*.62,0,0],accent);
    }
  }
  function waterline(paint: number) {
    a.add([2*x+.025,.09,2*z+.025],[0,-y*.48,0],paint);
  }
  function rim(paint: number) {
    for (const side of [-1,1]) {
      a.add([.06,.014,Math.max(.1,2*z-.3)],[side*(x-.15),top,0],paint);
      a.add([Math.max(.1,2*x-.3),.014,.06],[0,top,side*(z-.15)],paint);
    }
  }
  function caution() {
    for (const side of [-1,1]) {
      for (let i = -z+.23; i < z-.14; i += .48) {
        a.add([.23,.017,.11],[side*(x-.2),top,i],accent,.34);
      }
    }
  }
  function roofRoundel() {
    const r = Math.min(x,z)*.55;
    a.hoop(r,.045,[0,top,0],accent,true);
    for (const sign of [-1,1]) a.add([r*.15,.023,r*.5],[sign*r*.2,top,0],accent);
  }
  function cabin(paint: number) {
    // Keep the sides and glazing within the body's footprint so a player sees the same lip they can catch.
    for (const side of [-1,1]) {
      a.add([2*x+.012,.10,2*z+.012],[0,y*.64,0],paint);
      for (let i = -z+.55; i < z-.1; i += 1.08) {
        a.add([.022,Math.min(y*.58,.6),.75],[side*x,0,i],glass);
        a.add([.035,.045,.72],[side*(x+.003),y*.13,i],0xa7d1ca);
      }
      a.add([x*.65,y*1.07,.024],[side*x*.5,-y*.13,z],ink);
      a.add([x*.51,y*.38,.028],[side*x*.5,y*.09,z+.005],glass);
    }
  }

  switch (obj.art) {
    case 'palm': {
      const crown: [number,number,number] = [.10,y*.86,0];
      a.rod([-.10,-y,0],crown,x*.77,0x97694f);
      for (let i=0;i<7;i++) a.drum(x*.88,x*.71,.07,[-.10+i*.028,-y+.25+i*.39,0],0xad8763,9);
      for (let i=0;i<10;i++) {
        const angle=i*2.40, length=1.35+i%3*.27;
        const dx=Math.cos(angle)*length, dz=Math.sin(angle)*length;
        a.leaf(crown,[crown[0]+dx*.48,y*1.15,dz*.48],[crown[0]+dx,y*.71,dz],.24,i%2 ? 0x5c964f : 0x91b862);
        a.rod(crown,[crown[0]+dx*.56,y*1.06,dz*.56],.023,0xb6c674);
      }
      for (let i=0;i<3;i++) a.oval([.13,.17,.13],[.09*Math.cos(i*2),y*.75,.09*Math.sin(i*2)],0x76563f);
      break;
    }
    case 'lava-vent':
      a.drum(x*.72,x*1.05,y*1.7,[0,-y*.15,0],0x564d4e,8);
      a.drum(x*.58,x*.68,.13,[0,y*.76,0],0xfb8d42,8);
      a.drum(x*.29,x*.40,.05,[.05,y*.99,0],0xffd17a,8);
      break;
    case 'beacon':
      a.drum(x*.98,x*1.15,.48,[0,-y+.24,0],0xe89552,8);
      for (const side of [-1,1]) {
        a.rod([side*x,-y,0],[side*x*.44,y,0],.049,0x677d88);
        for (let i=0;i<4;i++) {
          const y0=-y+i*.61;
          a.rod([side*x*.85,y0,0],[-side*x*.67,y0+.61,0],.035,0xf2b16c);
        }
      }
      a.oval([.18,.19,.17],[0,y,0],0xffab4a);
      for(let i=0;i<3;i++) a.add([.99-i*.20,.055,.09],[0,y-.22-i*.29,0],0xb9d0d0);
      break;
    case 'cockpit':
      a.oval([x,y*.96,z],[0,-y*.07,0],0xe7dfc6);
      a.oval([x*.85,y*.70,z*.66],[0,y*.12,-z*.20],glass);
      a.rod([0,y*.73,-z*.57],[0,y*.83,z*.03],.045,ivory);
      break;
    case 'tail-fin':
      for (let i=0;i<9;i++) {
        const t=(i+.5)/9, depth=(1-t*.75)*2*z;
        a.add([x*1.65,2*y/9+.012,depth],[0,-y+2*y*t,z-depth*.5],i===6 ? ivory : 0xca564a);
      }
      break;
    case 'wheelhouse':
      cabin(ivory);
      a.add([2*x+.07,.13,2*z+.07],[0,y,0],0xd9875e);
      for (const side of [-1,1]) {
        a.add([x*.75,.56,.045],[side*x*.49,y*.24,z],glass);
        a.add([x*.71,.064,.055],[side*x*.49,y*.42,z+.02],0xa1c4bf);
      }
      a.drum(.16,.18,.75,[x*.61,y+.38,-z*.48],ink);
      a.rod([-x*.46,y,-z*.24],[-x*.46,y+1.33,-z*.24],.041,steel);
      a.rod([-x*.88,y+1.16,-z*.24],[-x*.04,y+1.16,-z*.24],.04,ivory);
      a.hoop(.27,.075,[-x*.49,-y*.19,z+.025],0xf29657);
      break;
    case 'sandbar':
    case 'basalt':
    case 'floe':
    case 'radio-floe':
    case 'ridge':
    case 'snow-shelf': {
      const icy = !['sandbar','basalt'].includes(obj.art);
      const crust = icy ? 0xd7f8f4 : obj.art === 'sandbar' ? 0xf7cc86 : 0x98aa6d;
      for (let i = 0; i < 7; i++) {
        const dx = Math.sin((i+1)*4.7 + obj.id) * x*.82;
        const dz = Math.cos((i+2)*5.1 + obj.id) * z*.78;
        a.add([.16+i%3*.055,.014,.07],[dx,top,dz],crust,i*.7);
      }
      for (const side of [-1,1]) {
        a.rod([-x+.12,-.03,side*z],[-x*.3,-y*.45,side*z],.045,crust);
        a.rod([-x*.3,-y*.45,side*z],[x*.42,-y*.20,side*z],.045,crust);
      }
      if (icy) {
        for(let i=0;i<4;i++) {
          const side = i%2 ? 1 : -1, dz = Math.sin(i*2.8+obj.id)*z*.68;
          a.drum(.22,.055,.43,[side*(x-.15),-y-.11,dz],0x79bfce,5);
        }
        const lengths = [[-.40,.14,.12,.45],[.14,-.30,.62,-.18],[-.77,-.46,-.46,-.29]] as const;
        for (const [sx,sz,ex,ez] of lengths) {
          const mx=(sx+ex)*x/2+.12, mz=(sz+ez)*z/2-.17;
          a.rod([sx*x,top,sz*z],[mx,top,mz],.029,0x80bcbf);
          a.rod([mx,top,mz],[ex*x,top,ez*z],.022,0x88c4c8);
          a.rod([mx,top,mz],[mx-.35,top,mz+.09],.016,0x9ac7c4);
        }
      } else if (obj.art === 'basalt') {
        for(let i=0;i<9;i++) {
          const r=.18+i%3*.08, dx=Math.sin(i*2.9+.8)*x*.88, dz=Math.cos(i*2.17+.7)*z*.89;
          a.oval([r,.09,r*.78],[dx,top-.012,dz],i%3 ? 0x839556 : 0x696653);
        }
      } else if (obj.art === 'sandbar') {
        for(let i=0;i<5;i++) {
          const tx=x*.71*Math.sin(i*2.3), tz=z*.62*Math.cos(i*1.8);
          a.oval([.14,.043,.10],[tx,top,tz],0xffdfaa);
          a.rod([tx,top,tz],[tx+.20,top+.15,tz-.04],.025,0x96a36a);
        }
      }
      if (obj.art === 'radio-floe') {
        a.hoop(.65,.037,[0,top,0],0x8ababc,true);
        a.add([.085,.013,1.15],[0,top,0],0x8ababc);
      }
      break;
    }
    case 'breaker':
      for (let i = -x+.45; i < x; i += .8) a.oval([.5,.24,.32],[i,y-.19,0],0xcbefde);
      break;
    case 'fuselage':
      for(const side of [-1,1]) {
        for(let i=-5;i<=4.5;i+=.74) a.oval([.035,.16,.18],[side*x,0,i],glass);
        a.add([.04,.13,z*1.40],[side*x,-y*.41,0],0xd65d49);
      }
      for (const side of [-1,1]) {
        a.add([.12,.02,1.4*z],[side*x*.80,top,0],0xd5694f);
        a.add([.24,.018,z*.29],[side*x*.38,top,-z*.77],glass,side*.16);
      }
      break;
    case 'wing':
    case 'tailplane':
      for (const sign of [-1,1]) {
        a.add([x*.16,.026,2*z-.04],[sign*x*.76,top,0],0xd4664e);
        a.add([x*.07,.026,2*z-.04],[sign*x*.56,top,0],ink);
      }
      if (obj.art === 'wing') {
        const nacelleX = -Math.sign(obj.center[0])*.45;
        a.oval([.53,.48,1.4],[nacelleX,-y-.24,-.32],0xbc5847);
        a.oval([.42,.37,.06],[nacelleX,-y-.24,-1.60],ink);
        a.oval([.13,.13,.08],[nacelleX,-y-.24,-1.65],ivory);
        a.oval([.38,.34,.08],[nacelleX,-y-.24,.99],0x484b53);
        a.oval([.16,.15,.09],[nacelleX,-y-.24,1.05],0x9f8873);
      }
      roofRoundel();
      break;
    case 'luggage':
    case 'fishbox':
    case 'freight':
    case 'brick-cart':
    case 'hook-load':
      bands();
      if (obj.art === 'luggage') {
        a.rod([-.13,y,-.035],[-.13,y+.12,-.035],.027,ink);
        a.rod([-.13,y+.12,-.035],[.13,y+.12,-.035],.027,ink);
        a.rod([.13,y+.12,-.035],[.13,y,-.035],.027,ink);
      } else if (obj.art === 'hook-load') {
        a.hoop(.20,.045,[0,y+.14,0],ink);
        a.rod([0,y+.28,0],[0,y+4,0],.027,ink);
      } else {
        for (let i = -x+.2; i < x-.1; i += .29) a.add([.065,y*1.4,2*z+.023],[i,0,0],edge);
      }
      break;
    case 'hull':
    case 'stern':
    case 'barge':
      if (obj.art !== 'hull') { waterline(accent); rim(ivory); }
      else {
        for(const side of [-1,1]) {
          a.rod([side*x*.54,-y*.16,-z],[side*x*.89,-y*.16,-z*.76],.058,ivory);
          a.rod([side*x*.89,-y*.16,-z*.76],[side*x*.97,-y*.16,-z*.52],.058,ivory);
          a.rod([side*x*.97,-y*.16,-z*.52],[side*x*.97,-y*.16,z*.67],.058,ivory);
          a.hoop(.30,.072,[side*x*.84,top+.052,z*.41],0xe87850,true);
          a.rod([side*x*.83,top,z*.52],[side*x*.65,top+2.5,z*.12],.075,0x7b7471);
        }
        a.rod([-x*.65,top+2.5,z*.12],[x*.65,top+2.5,z*.12],.09,0x5d6262);
        for(let i=-3;i<=3;i++) a.rod([i*.54,top+2.48,z*.12],[i*.45,top+.58,z*.43],.021,0x9c8c77);
      }
      for (const side of [-1,1]) for (const end of [-1,1]) {
        const p: [number,number,number] = [side*(x-.28),top+.013,end*(z-.4)];
        a.drum(.16,.16,.035,p,ink,12);
      }
      break;
    case 'shark':
      // The mesh is centered inside the authoritative swept bite volume, with its snout toward positive X.
      a.oval([x*.83,y*.76,z*.92],[x*.10,-y*.08,0],0x577781);
      a.oval([x*.66,y*.40,z*.88],[x*.20,-y*.30,0],0xc4d9ca);
      a.fin(x*.32,y*.85,z*.23,[-x*.05,y*.52,0],0x486b78);
      a.fin(x*.25,y*1.38,z*.22,[-x*.80,.02,0],0x456876);
      for (const side of [-1,1]) {
        a.oval([.078,.078,.034],[x*.62,y*.15,side*z*.75],0x272e38);
        a.oval([.022,.022,.01],[x*.64,y*.18,side*z*.79],ivory);
        a.rod([x*.46,-y*.20,side*z*.84],[x*.80,-y*.24,side*z*.55],.025,0x503e4b);
      }
      break;
    case 'lift-car':
    case 'tram':
    case 'gondola':
    case 'train':
    case 'railcar':
    case 'shuttle':
      cabin(accent); rim(ivory);
      if (obj.art === 'lift-car') {
        const sideId = obj.id === 1 ? -1 : 1;
        a.add([.22,.018,1.7],[0,top,0],accent);
        for (const sign of [-1,1]) a.rod([0,top,sideId*.87],[sign*.46,top,sideId*.32],.085,accent);
        for (const side of [-1,1]) {
          a.rod([side*x*.91,top,-z*.90],[side*x*.91,top+6,-z*.90],.035,steel);
          a.drum(.19,.19,.06,[side*x*.91,top,-z*.90],accent);
        }
      } else if (obj.art === 'gondola' || obj.art === 'tram') {
        for(const side of [-1,1]) {
          a.rod([side*(x-.12),top,-z+.18],[side*x*.64,top+1.8,-z+.18],.068,edge);
          a.rod([side*x*.64,top+1.8,-z+.18],[0,top+2.18,-z+.18],.065,accent);
        }
        a.rod([0,top+2.18,-z+.18],[0,top+3.7,-z+.18],.042,ink);
      } else if (obj.art === 'train') {
        for (const side of [-1,1]) a.add([.34,.18,.06],[side*x*.69,-y*.14,z],ivory);
      } else if (obj.art === 'shuttle') {
        roofRoundel();
        a.add([x*.93,.018,.64],[0,top,-z*.69],glass);
      }
      break;
    case 'track':
      for (let i = -z+.2; i < z; i += .61) a.add([1.7*x,.024,.14],[0,top,i],0x715a4f);
      for (const side of [-1,1]) a.add([.10,.055,1.99*z],[side*x*.62,top+.018,0],0xb8bdac);
      break;
    case 'platform':
    case 'gantry':
    case 'service-deck':
    case 'gatehouse':
    case 'spillgate':
    case 'press-head':
    case 'press-bed':
    case 'transfer-table':
      caution();
      if (obj.art === 'gantry' || obj.art === 'service-deck') truss();
      if (obj.art === 'press-head') {
        a.drum(x*.41,x*.56,.42,[0,top+.19,0],0xafaa98);
        a.drum(x*.23,x*.24,3.6,[0,top+1.9,0],steel);
      }
      if (obj.art === 'gatehouse') {
        for(const side of [-1,1]) a.add([.14,.025,z*1.60],[side*x*.67,top,0],0x85938b);
      }
      for (const side of [-1,1]) a.add([2*x-.07,.07,2*z+.024],[0,side*y*.57,0],ink);
      break;
    case 'furnace-door':
    case 'brick-bed':
      for(let i=-x+.23;i<x;i+=.45) a.add([.17,.045,2*z-.2],[i,-y,0],0xff9951);
      for (let i=-z+.22;i<z;i+=.53) a.add([1.90*x,.016,.034],[0,top,i],0x71514b);
      rim(ink);
      break;
    case 'turbine-arm':
    case 'feed-arm':
    case 'boom':
    case 'signal-bar':
    case 'paddle':
      for (let i=-x+.2;i<x-.08;i+=.58) a.add([.21,2*y+.021,2*z+.021],[i,0,0],accent);
      a.drum(Math.min(z,.23),Math.min(z,.23),.032,[0,top,0],steel);
      break;
    case 'float':
    case 'gondola-decor':
      rim(ivory);
      for (const side of [-1,1]) for (let i=-z+.3;i<z;i+=.61) {
        const color = [0xe98578, 0xffdc7b, 0x8ccdc1][(Math.floor(i+z)+obj.id)%3]!;
        a.oval([.08,y*.91,.35],[side*x,-.03,i],color);
      }
      const badge=[0xf8d582,0x63ab9e,0xaa79a6][(obj.id-1)%3]!;
      for(const side of [-1,1]) a.oval([.12,y*.79,.38],[side*x,-y*.04,z*.74],badge);
      a.drum(Math.min(x,z)*.51,Math.min(x,z)*.51,.015,[0,top,0],badge,8);
      a.hoop(Math.min(x,z)*.60,.06,[0,top,0],ivory,true);
      break;
    case 'turntable': {
      const r=Math.min(x,z)*.89;
      a.drum(r,r,.026,[0,top,0],0x36354c,48);
      for(let i=1;i<6;i++) a.hoop(r*(.38+i*.10),.017,[0,top+.018,0],0x63566d,true);
      a.drum(r*.34,r*.34,.029,[0,top+.013,0],0xd6868e,32);
      a.drum(.09,.09,.05,[0,top+.036,0],ivory);
      a.rod([x*.86,top+.08,-z*.72],[x*.47,top+.08,z*.08],.045,accent);
      break;
    }
    case 'drum-pad': {
      const r=Math.min(x,z)*.84, paint=[0xd5809c,0x86c9ba,0xefb977,0xaa91c8][(obj.id-2+4)%4]!;
      a.drum(r,r,.026,[0,top,0],paint,28);
      a.hoop(r,.06,[0,top+.015,0],ink,true);
      a.hoop(r*.7,.022,[0,top+.025,0],ivory,true);
      for(const sign of [-1,1]) a.add([.26,.16,.05],[sign*x*.65,0,z],ivory);
      break;
    }
    case 'docking-collar':
    case 'reflector':
    case 'turbine-base':
    case 'cider-vat':
      for (const scale of [.83,.66,.30]) a.hoop(Math.min(x,z)*scale,.032,[0,top,0], scale===.66 ? ink : accent,true);
      a.add([.075,.015,Math.min(x,z)*.65],[0,top,Math.min(x,z)*.48],ivory);
      waterline(ink);
      break;
    default:
      rim(accent);
      for (let i=-z+.25;i<z;i+=.56) a.add([1.92*x,.014,.035],[0,top,i],ink);
  }
  a.attach(parent);
}
