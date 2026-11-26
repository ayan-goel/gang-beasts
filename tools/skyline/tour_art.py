"""Paint the tour's repeating surface prints and seamless panoramic surroundings."""
from math import cos, pi, sin
from pathlib import Path
from random import Random
import json
from PIL import Image, ImageColor, ImageDraw

ROOT = Path(__file__).resolve().parents[2] / 'web/src/rendering/tour'
RGB = ImageColor.getrgb
W, H = 1536, 768
NATURAL = {'atoll', 'polar', 'alpine', 'reservoir', 'orchard', 'railway', 'observatory'}
URBAN = {'port', 'harbor', 'towers', 'works', 'steel', 'parade', 'kiln', 'metro'}
NIGHT = {'space', 'concert', 'polar', 'observatory', 'harbor'}


def mix(first, second, amount):
    return tuple(round(a + (b-a)*amount) for a, b in zip(first, second))


def panorama(p, rng):
    land = p['land']
    sky, haze, depth, ink, accent = (RGB(p[k]) for k in ('sky', 'haze', 'distance', 'ink', 'accent'))
    image = Image.new('RGB', (W, H))
    draw = ImageDraw.Draw(image)
    stops = [(0, sky), (205, mix(sky, haze, .36)), (365, haze),
             (505, mix(depth, haze, .55)), (767, mix(ink, depth, .52))]
    for (y0, c0), (y1, c1) in zip(stops, stops[1:]):
        for y in range(y0, y1+1):
            draw.line((0, y, W, y), fill=mix(c0, c1, (y-y0)/(y1-y0)))
    if land in NIGHT:
        for _ in range(675):
            x, y = rng.randrange(W), rng.randrange(20, 746 if land == 'space' else 345)
            r = rng.choices([0, 1, 2], [70, 25, 5])[0]
            draw.ellipse((x-r, y-r, x+r, y+r), fill=mix(sky, (255, 239, 203), rng.uniform(.18, .75)))
    sun_x = rng.randrange(285, 1290)
    y, r = (289, 50) if land == 'space' else (305, 17)
    draw.ellipse((sun_x-r, y-r, sun_x+r, y+r), fill=mix(haze, (255, 239, 183), .74))
    if land == 'space':
        for dy in range(-26, 28, 7):
            span = int((r*r-dy*dy)**.5)
            draw.arc((sun_x-span, y+dy-8, sun_x+span, y+dy+8), 0, 180, fill=mix(depth, haze, .2), width=2)
        draw.arc((sun_x-r*2, y-10, sun_x+r*2, y+15), 0, 300, fill=mix(haze, (240, 192, 160), .8), width=5)
        return image
    for _ in range(45):
        x, y = rng.randrange(W), rng.randrange(175, 365)
        span = rng.randrange(28, 128)
        rise = rng.randrange(3, 14)
        color = mix(haze, (255, 242, 215), rng.uniform(.13, .45))
        for ox in (x-W, x, x+W):
            draw.polygon([(ox, y), (ox+span//3, y-rise//2), (ox+span//2, y-rise),
                          (ox+span*2//3, y-rise+1), (ox+span*3//4, y-3), (ox+span, y)], fill=color)
    if land == 'polar':
        for stream in range(3):
            for x in range(0, W, 4):
                top = 170 + stream*28 + 36*sin(2*pi*x/W + stream)
                draw.line((x, top, x+1, top+12+7*cos(x*6*pi/W)), fill=mix(sky,(130,236,189),.28+stream*.06))
    for row, base in enumerate((383, 415, 453)):
        shade = mix(haze, depth, .32 + row*.21)
        phases = [rng.uniform(0, 2*pi) for _ in range(3)]
        heights = []
        for x in range(0, W+1, 8):
            a = x*2*pi/W
            peaks = sum(cos(k*a+offset)*weight for k, offset, weight in zip((3,9,21), phases, (23,15,6)))
            if land in {'polar','alpine','atoll','reservoir','observatory'}:
                peaks += 32*abs(sin(5*a+phases[0]))
            if land == 'flight':
                peaks = 11*sin(5*a+phases[2]) + 19*abs(cos(24*a+phases[0]))
            heights.append((x, int(base-peaks)))
        draw.polygon([(0,H), *heights, (W,H)], fill=shade)
        if land in {'alpine','polar'}:
            for i in range(2, len(heights)-3):
                x, y = heights[i]
                if y <= min(heights[i-1][1], heights[i+1][1]) and y < base-14:
                    draw.polygon([heights[i-2], (x,y), heights[i+2], (x+4,y+8),(x-2,y+5)], fill=mix(shade,(228,243,236),.65))
    if land in URBAN:
        for band in range(2):
            x = 0
            while x < W:
                w = rng.randrange(17, 49)
                floor = 412 + band*44
                h = rng.randrange(28, 120 if land == 'towers' else 88)
                paint = mix(depth, ink, .13+band*.25)
                y = floor-h
                draw.rectangle((x, y, x+w, floor), fill=paint)
                draw.rectangle((x, y, x+w, y+3), fill=mix(paint,haze,.25))
                if land in {'steel','kiln','works','port'}:
                    draw.rectangle((x+4,y-12,x+10,y),fill=paint)
                for wx in range(x+3, x+w-4, 8):
                    for wy in range(y+8, floor-3, 10):
                        if rng.randrange(4) == 0:
                            draw.rectangle((wx,wy,wx+3,wy+2), fill=mix(paint,accent,.5))
                x += w
    if land in NATURAL - {'polar', 'alpine'}:
        for _ in range(175):
            x, y = rng.randrange(W), rng.randrange(418, 479)
            h = rng.randrange(5,20)
            draw.polygon([(x-h//3,y), (x,y-h), (x+h//3,y)],fill=mix(depth,ink,.21))
    return image


def surface(p, rng):
    n = 384
    base, ink, accent = (RGB(p[k]) for k in ('deck','ink','accent'))
    land = p['land']
    if land in {'atoll','polar','alpine'}:
        base, ink = (244,246,238), (154,180,176)
    image = Image.new('RGB',(n,n),base)
    draw = ImageDraw.Draw(image)
    if land in {'polar','alpine','atoll','reservoir'}:
        for _ in range(95):
            x,y = rng.randrange(n),rng.randrange(n)
            size = rng.randrange(3,15)
            color = mix(base,ink,rng.uniform(.035,.125))
            draw.polygon([(x,y),(x+size,y-2),(x+size*2,y+2),(x+3,y+6)],fill=color)
        if land in {'polar','alpine'}:
            for x,y in [(58,43),(227,148),(106,305)]:
                draw.line([(x,y), (x+16,y+10),(x+8,y+16),(x+39,y+27)],fill=mix(base,ink,.30),width=2)
                draw.line([(x+8,y+16),(x-3,y+25),(x+1,y+34)], fill=mix(base,ink,.20),width=1)
    elif land in {'harbor','orchard','circus','pier','railway'}:
        for i in range(6):
            y = i*64
            color = mix(base, ink, (i%3)*.025)
            draw.rectangle((0,y,n,y+63),fill=color)
            draw.line((0,y,n,y),fill=mix(base,ink,.34),width=2)
            x = (i*73+19)%n
            draw.line((x,y,x,y+64),fill=mix(base,ink,.23),width=2)
            for k in (-7,7):
                draw.ellipse((x+k-1,y+7,x+k+1,y+9),fill=mix(base,ink,.55))
            for _ in range(19):
                x0, y0 = rng.randrange(n-35), y+rng.randrange(4,58)
                draw.line((x0,y0,x0+rng.randrange(4,30),y0), fill=mix(base,ink,.085),width=2)
    elif land == 'metro':
        for i in range(6):
            for j in range(6):
                x,y = i*64,j*64
                draw.rectangle((x+2,y+2,x+63,y+63),fill=mix(base,ink,rng.choice([.03,.045,.075])))
                draw.line((x,y,x+64,y),fill=mix(base,ink,.22),width=2)
                draw.line((x,y,x,y+64),fill=mix(base,ink,.22),width=2)
    else:
        for i in range(2):
            for j in range(3):
                x,y=i*192,j*128
                c=mix(base,ink, ((i+j)%4)*.016)
                draw.rectangle((x+1,y+1,x+191,y+127),fill=c)
                draw.line((x,y+128,x,y,x+192,y),fill=mix(base,ink,.27),width=2)
                for dx in (9,181):
                    for dy in (8,118):
                        draw.ellipse((x+dx,y+dy,x+dx+2,y+dy+2),fill=mix(base,ink,.36))
                if land in {'orbital','space','concert','observatory','windfarm'}:
                    draw.line((x+67,y+7,x+126,y+7),fill=mix(base,accent,.6),width=3)
    for _ in range(2200):
        x,y=rng.randrange(n),rng.randrange(n)
        shade=mix(image.getpixel((x,y)),rng.choice([ink,(255,255,245)]),rng.uniform(.013,.07))
        draw.line((x,y,x+rng.randrange(1,4),y),fill=shade)
    return image


for i,(stage,palette) in enumerate(json.loads((ROOT/'palettes.json').read_text()).items()):
    random = Random(861+i)
    panorama(palette,random).save(ROOT/'art'/f'{stage}-sky.webp',quality=83)
    surface(palette,random).save(ROOT/'art'/f'{stage}-floor.webp',quality=82)
