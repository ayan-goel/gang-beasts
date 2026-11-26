"""Bake the Studio 08 roof paint as a single mipmapped texture."""
from pathlib import Path
from random import Random
from PIL import Image, ImageDraw

rng = Random(8)
size = 1024
image = Image.new('RGB', (size,size), '#81a69d')
draw = ImageDraw.Draw(image)
# Offset membrane rolls and a sun-faded landing mark remain visible in the tight two-player shot.
for row, y in enumerate(range(0,size,96)):
    for x in range(-200 + row%2*96,size,195):
        variation = rng.randrange(-4,5)
        draw.rectangle((x,y,x+193,y+93), fill=tuple(v+variation for v in (120,157,148)))
        for k in [8,185]:
            draw.ellipse((x+k,y+5,x+k+2,y+7),fill='#6c9189')
for x in (20,1004):
    draw.line((x,16,x,1008),fill='#d6cba1',width=5)
for y in (20,1004):
    draw.line((16,y,1008,y),fill='#d6cba1',width=5)
for x,y in [(80,80),(944,80),(944,944),(80,944)]:
    draw.ellipse((x-7,y-7,x+7,y+7),outline='#d6cba1',width=2)
    draw.line((x-18,y,x+18,y),fill='#d6cba1',width=2)
    draw.line((x,y-18,x,y+18),fill='#d6cba1',width=2)
# Painted test-card bullseye with paired bean eyes and registration ticks.
cx,cy = 512,512
for radius,width in [(158,3),(148,9),(124,2)]:
    draw.ellipse((cx-radius,cy-radius,cx+radius,cy+radius),outline='#a9bbaa',width=width)
for x,y in [(cx-33,cy),(cx+33,cy-6)]:
    draw.rounded_rectangle((x-14,y-33,x+14,y+30),radius=14,fill='#a9bbaa')
for d in (-1,1):
    draw.line((cx+d*129,cy,cx+d*178,cy),fill='#bcc5ac',width=4)
    draw.line((cx,cy+d*129,cx,cy+d*178),fill='#bcc5ac',width=4)
# An equipment-bay stencil gives the deck its station number at camera distances
# where the seams and fasteners are already smaller than a pixel.
digits = ('01110100011001110101110011000101110', '01110100011000101110100011000101110')
for column, glyph in enumerate(digits):
    for index, filled in enumerate(glyph):
        if filled == '1':
            x = 735 + column*70 + (index%5)*10
            y = 196 + (index//5)*10
            draw.rectangle((x,y,x+8,y+8),fill='#c0cab1')
for x in range(46,280,28):
    for y in (36,975):
        draw.polygon([(x,y),(x+14,y),(x+5,y+12),(x-9,y+12)],fill='#d8c898')
for _ in range(4700):
    x,y = rng.randrange(size),rng.randrange(size)
    a = image.getpixel((x,y))
    v = rng.choice([-9,-5,6,11])
    draw.line((x,y,x+rng.randrange(1,5),y),fill=tuple(max(0,min(255,c+v)) for c in a))
image.save(Path(__file__).resolve().parents[2]/'web/src/rendering/art/studio-deck.webp',quality=85)
