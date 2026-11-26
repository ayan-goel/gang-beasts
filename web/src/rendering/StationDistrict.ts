import { SkylineDetails } from './SkylineDetails';
import type { CameraObstacles } from './CameraObstacles';
import type { Object3D } from 'three';

const towers = [
  [-22,-20,8,23], [-10,-28,9,18], [2,-31,8,25], [16,-22,8,20], [26,-16,8,25],
  [-26,-3,7,18], [25,1,7,17], [-26,17,9,25], [23,21,8,20], [-9,27,8,21],
  [4,31,8,17], [-38,-33,13,27], [35,-39,14,30], [-1,-46,12,34],
] as const;

function stationMarquee(art: SkylineDetails, x: number, roof: number, z: number) {
  const ink = 0x493347;
  const y = roof + 3.7;
  for (const leg of [-3, 3]) {
    art.rod([x + leg, roof, z], [x + leg, y, z], .13, ink);
    art.rod([x + leg, roof, z - 1.2], [x + leg, y - .5, z], .08, ink);
  }
  art.add([8.3, 2.3, .22], [x, y, z], ink);
  art.add([8, 2, .09], [x, y, z + .13], 0xac4849);
  // One district mesh carries both the frame and the bulb lettering.
  const letters = ['111101101101111', '100111111111001', '010101111101101', '111010010010111', '110101110101101'];
  letters.forEach((glyph, index) => {
    for (let cell = 0; cell < glyph.length; cell++) {
      if (glyph[cell] === '0') continue;
      const offset = -3.38 + index * 1.33 + (index > 1 ? .59 : 0);
      art.add([.24, .22, .06], [x + offset + cell % 3 * .31, y + .62 - Math.floor(cell / 3) * .31, z + .21], 0xffdda1);
    }
  });
}

export function dressDistrict(root: Object3D, obstacles: CameraObstacles) {
  const art = new SkylineDetails();
  const walls = [0xb66b64,0x977389,0xc68d77,0x97767c,0xaa776d];
  towers.forEach(([x,z,w,h], i) => {
    const top = h-24;
    obstacles.fixed([x,top-h/2,z],[w/2,h/2,w/2]);
    art.add([w,h,w],[x,top-h/2,z],walls[i%walls.length]!);
    art.add([w+.18,.24,w+.18],[x,top,z],0x58495a);
    for (let y=top-1.1; y>-21; y-=2.25) {
      for (const offset of [-.33,-.10,.13,.34]) {
        const pane = Math.abs(Math.round(y*5+offset*80+i))%5===0 ? 0xffd391 : 0x615767;
        for (const side of [-1,1]) {
          art.add([w*.135,.85,.06],[x+offset*w,y,z+side*w/2],pane);
          art.add([.06,.85,w*.135],[x+side*w/2,y,z+offset*w],pane);
        }
      }
    }
    const radius=w*.16;
    obstacles.fixed([x,top+1.85,z],[radius+.2,1.85,radius+.2]);
    if (i === 2) {
      stationMarquee(art, x, top, z);
      obstacles.fixed([x, top + 3.7, z], [4.15, 1.15, .22]);
    } else if (i%3===0) {
      for (const dx of [-.7,.7]) for (const dz of [-.7,.7])
        art.rod([x+dx,top,z+dz],[x+dx*.8,top+1.6,z+dz*.8],.085,0x594957);
      art.drum(radius,radius,1.6,[x,top+2.15,z],0xd28d71);
      art.drum(radius+.035,radius+.035,.11,[x,top+1.72,z],0x574957);
      art.drum(.04,radius+.19,.65,[x,top+3.27,z],0x675668);
    } else {
      art.add([radius*2,1.2,radius*2],[x,top+.6,z],0xb99588);
      art.rod([x,top+1.1,z],[x,top+3.5,z],.06,0x4b4051);
      for (let bar=0;bar<3;bar++) {
        const y=top+2.3+bar*.42;
        art.add([1.95-bar*.36,.075,.08],[x,y,z],0x584957,.3);
      }
    }
  });
  art.attach(root);
}
