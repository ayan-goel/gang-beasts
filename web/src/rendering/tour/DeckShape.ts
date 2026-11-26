import { BufferGeometry, Float32BufferAttribute } from 'three';
import type { ArenaObject, ArenaPoint } from '../SkylineView';

type Point = [number,number];
const ice = new Set(['radio-floe','floe','snow-shelf','ridge']);
const land = new Set(['basalt','sandbar']);
const vessels = new Set(['hull','stern','fuselage','wing','tailplane']);

// Keep the central load-bearing face level. Shallow perimeter cuts and faceted undersides
// provide a silhouette without introducing a different walkable height from the physics box.
export function deckShape(obj: ArenaObject): BufferGeometry | undefined {
  const tag=obj.art ?? '';
  if (!ice.has(tag) && !land.has(tag) && !vessels.has(tag)) return;
  const [w,h,d] = obj.half;
  let outline: Point[];
  if (tag === 'fuselage' || tag === 'hull') {
    outline = [[-.57*w,-d], [.57*w,-d],[.89*w,-d+.26],[w,-d*.62],[w,d*.67],[.94*w,d-.23],
      [.75*w,d],[-.75*w,d],[-.94*w,d-.23],[-w,d*.67],[-w,-d*.62],[-.89*w,-d+.26]];
  } else if (tag === 'wing' || tag === 'tailplane') {
    const side = Math.sign(obj.center[0]);
    outline = [[-w+.16,-d],[w-.16,-d],[w,-d+.22],[w,d-(side>=0?.36:.09)],[w-.22,d],[-w+.22,d],[-w,d-(side<=0?.36:.09)],[-w,-d+.22]];
  } else {
    const cut = Math.min(.54, w*.21, d*.22);
    outline = [[-w+cut,-d],[-w*.24,-d+.045],[w*.42,-d+.02],[w-cut*1.2,-d],[w,-d+cut*.96],
      [w-.04,-d*.34],[w,d*.56],[w-.08,d-cut],[w-cut*.79,d],[w*.20,d-.035],
      [-w*.44,d],[-w+cut,d-.06],[-w,d-cut*1.21],[-w+.04,d*.24],[-w,-d*.47],[-w,-d+cut*.94]];
  }
  const verts:number[]=[];
  const point=(xz:Point,y:number,scale:number):ArenaPoint=>[xz[0]*scale,y,xz[1]*scale];
  const tri=(a:ArenaPoint,b:ArenaPoint,c:ArenaPoint)=>verts.push(...a,...b,...c);
  for (let i=0;i<outline.length;i++) {
    const next=(i+1)%outline.length;
    const a=outline[i]!, b=outline[next]!;
    const ta=point(a,h,1), tb=point(b,h,1);
    const ma=point(a,h*.10,.99), mb=point(b,h*.10,.99);
    const keel = tag==='hull'||tag==='fuselage' ? .79 : .94;
    const ba=point(a,-h,keel), bb=point(b,-h,keel);
    tri([0,h,0],tb,ta);
    tri(ta,tb,ma); tri(tb,mb,ma);
    tri(ma,mb,ba); tri(mb,bb,ba); tri([0,-h,0],ba,bb);
  }
  const geom=new BufferGeometry();
  geom.setAttribute('position',new Float32BufferAttribute(verts,3));
  geom.setAttribute('uv',new Float32BufferAttribute(new Float32Array(verts.length/3*2),2));
  geom.computeVertexNormals();
  return geom;
}
