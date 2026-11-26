"""Paint the distant city and cloud bands onto one wrapping panorama."""
from pathlib import Path
from random import Random

from PIL import Image, ImageDraw

width, height = 1536, 768
rng = Random(108)
image = Image.new('RGB', (width, height))
draw = ImageDraw.Draw(image)
stops = [(0, (126, 157, 183)), (245, (250, 175, 157)), (368, (255, 210, 168)),
         (428, (235, 170, 153)), (530, (168, 127, 148)), (767, (111, 95, 121))]
for (top, start), (bottom, end) in zip(stops, stops[1:]):
    for y in range(top, bottom + 1):
        t = (y - top) / (bottom - top)
        color = tuple(round(a + (b - a) * t) for a, b in zip(start, end))
        draw.line((0, y, width, y), fill=color)

# Repeat edge-crossing cloud shapes at both ends so an orbit never reveals the seam.
for _ in range(30):
    x, y = rng.randrange(width), rng.randrange(230, 357)
    span, rise = rng.randrange(24, 125), rng.randrange(2, 9)
    for origin in (x - width, x, x + width):
        draw.polygon([(origin, y), (origin + span//4, y-rise//2),
                      (origin + span//3, y-rise), (origin+span//2, y-rise),
                      (origin + span*2//3, y-2), (origin + span, y)],
                     fill='#ffddbf')

for baseline, low, high, paint in [(412, 345, 390, '#dcaa9e'), (452, 370, 419, '#c79795')]:
    x = 0
    while x < width:
        span, roof = rng.randrange(12, 38), rng.randrange(low, high)
        draw.rectangle((x, roof, x+span, baseline), fill=paint)
        if rng.randrange(3) == 0:
            draw.rectangle((x+3, roof-4, x+span//2, roof), fill=paint)
            draw.line((x+5, roof-8, x+5, roof), fill=paint)
        x += span
    draw.rectangle((0, baseline, width, baseline+12), fill=paint)
image.save(Path(__file__).resolve().parents[2] / 'web/src/rendering/art/station-sky.webp', quality=87)
