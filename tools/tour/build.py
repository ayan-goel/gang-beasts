"""Author the tour in metres and seconds. Bake full manifests for the shared Rust engine."""
from math import atan2, pi
from pathlib import Path
import json

ROOT = Path(__file__).resolve().parents[2]
stages = []
X, Y, Z = [1, 0, 0], [0, 1, 0], [0, 0, 1]
DOWN = [0, -1, 0]


class Stage:
    def __init__(self, slug, name, theme, twist, water=None):
        self.slug, self.objects, self.motions = slug, [], []
        self.tour = dict(name=name, theme=theme, twist=twist, motions=self.motions)
        if water is not None:
            self.tour['water'] = water
        stages.append(self)

    def box(self, art, x, z, width, depth, top=0, height=.6, kind='deck', parent=None):
        height = {'fuselage':1.5,'lift-car':2.4,'railcar':1.5,'tram':1.5,'gondola':1.5,
                  'barge':1.0,'hull':1.0}.get(art, height)
        obj = len(self.objects) + 1
        self.objects.append(dict(id=obj, kind=kind, art=art, center=[x, round(top-height/2, 3), z],
                                 half=[width/2, height/2, depth/2], mass=8 if kind=='crate' else 0))
        if parent is not None:
            self.objects[-1]['parent'] = parent
        return obj

    def move(self, obj, kind, axis, amplitude, seconds, after):
        self.motions.append(dict(object=obj, kind=kind, axis=axis, amplitude=amplitude,
                                 period=round(seconds*60), start=round(after*60)))

    def manifest(self):
        # Choose opposed spots on different starting platforms wherever possible.
        platforms = [o for o in self.objects if o['kind']=='deck' and o['center'][1]+o['half'][1] == 0]
        candidates = []
        for platform in platforms:
            x, _, z = platform['center']
            w, _, d = platform['half']
            for dx, dz in [(-.46,.46),(.46,-.46),(-.46,-.46),(.46,.46),(0,0),(-.65,0),(.65,0),(0,.65),(0,-.65)]:
                px, pz = round(x+dx*w,2), round(z+dz*d,2)
                if all(o is platform or abs(px-o['center'][0])>o['half'][0]+.55 or
                       abs(pz-o['center'][2])>o['half'][2]+.55 or o['center'][1]+o['half'][1]<.1 for o in self.objects):
                    candidates.append((px,pz))
        spawns = []
        for target in [(-2,1.5),(2,-1.5),(-5,-3),(5,3),(-5,3),(5,-3),(0,-5),(0,5)]:
            choices = [p for p in candidates if all((p[0]-s[0])**2+(p[1]-s[1])**2>1.7**2 for s in spawns)]
            if not choices:
                raise ValueError(f'{self.slug}: no clear spawn at {target}')
            spawns.append(min(choices,key=lambda p:(p[0]-target[0])**2+(p[1]-target[1])**2))
        first = self.objects[0]
        x, y, z = first['center']
        first_event = min(m['start'] for m in self.motions)
        last_event = max(m['start'] for m in self.motions)
        return dict(schema_version=1,content_version=1,id=self.slug,fall_y=-10,
                    foundation=dict(center=[x,round(y-first['half'][1]-.12,3),z],half=[.25,.12,.25]),
                    schedule=dict(warn_tick=max(60,first_event-240),west_tick=first_event,
                                  east_tick=max(first_event+240,last_event),gust_tick=66*60,end_tick=88*60,gust_acceleration=3.5),
                    spawns=[dict(position=[x,0,z],yaw=round(atan2(x,z),3)) for x,z in spawns],
                    objects=self.objects,tour=self.tour)


stage = Stage('squall-island', 'Squall Island', 'island', 'The outer beaches slip under. Cross the rock bridge before the storm surge.', -0.64)
stage.box('basalt', 0, 0, 7, 8)
west_sandbar = stage.box('sandbar', -5.5, 0, 4, 6)
east_sandbar = stage.box('sandbar', 5.5, 1, 4, 7)
stage.box('jetty', 0, -5.5, 3, 3)
stage.box('palm', 2.2, -2.6, 0.48, 0.48, 3.4, 3.4, 'vent')
stage.box('lava-vent', -2, 2.7, 1.35, 1.4, 0.6, 0.6, 'vent')
stage.move(west_sandbar, 'sink', DOWN, 2.8, 15, 18)
stage.move(east_sandbar, 'roll', Z, 0.18, 6, 28)
stage.move(east_sandbar, 'sink', DOWN, 2.5, 15, 43)
breaker = stage.box('breaker', 0, -10, 9, 1, -0.5, 1.2, 'vent')
stage.move(breaker, 'slide', Z, 13, 12, 34)
stage.move(breaker, 'heave', Y, 1, 12, 34)

stage = Stage('cloud-courier', 'Cloud Courier', 'aviation', 'The cargo plane banks. Loose baggage slides toward the wing tips.')
fuselage = stage.box('fuselage', 0, 0, 4.6, 15)
west_wing = stage.box('wing', -4.9, 0.2, 5.2, 3.8, parent=fuselage)
east_wing = stage.box('wing', 4.9, 0.2, 5.2, 3.8, parent=fuselage)
stage.box('tailplane', 0, 6, 9, 1.6, 0.18, 0.18, 'panel', parent=fuselage)
stage.box('cockpit', 0, -5.65, 2.45, 2.3, 0.96, 0.96, 'vent', parent=fuselage)
stage.box('tail-fin', 0, 6.3, 0.31, 1.8, 1.8, 1.62, 'vent', parent=fuselage)
stage.move(fuselage, 'roll', Z, 0.16, 10, 16)
stage.move(west_wing, 'heave', Y, 0.95, 4, 34)
stage.move(east_wing, 'heave', DOWN, 0.95, 4, 34)
for x in [-1.4, 1.4]:
    stage.box('luggage', x, -3, 0.85, 0.75, 0.65, 0.65, 'crate')

stage = Stage('ice-radio', 'Ice Radio', 'polar', 'The ice field pulls apart. The smallest floes sink first.', -0.55)
stage.box('radio-floe', 0, 0, 5, 8)
stage.box('beacon', 0.95, -2.9, 0.5, 0.5, 2.6, 2.6, 'vent')
for x, z, direction, delay in [(-4.7, -2, -1, 14), (4.7, 1, 1, 19), (-3, 4, -1, 24)]:
    floe = stage.box('floe', x, z, 4.1, 3.9)
    stage.move(floe, 'sink', [direction, 0, 0], 3, 16, delay)
    stage.move(floe, 'sink', DOWN, 2.7, 16, delay + 20)

stage = Stage('night-trawler', 'Night Trawler', 'harbor', 'A shark breaches across the deck as the stern takes on water.', -0.65)
hull = stage.box('hull', 0, -2, 7, 9)
stern = stage.box('stern', 0, 4, 6.5, 3)
gangplank = stage.box('gangplank', -4, -1, 1, 6)
stage.move(hull, 'roll', Z, 0.1, 8, 12)
stage.move(stern, 'sink', DOWN, 2.2, 24, 39)
shark = stage.box('shark', -8, 1, 2.8, 1.25, -0.65, 1.2, 'vent')
stage.move(shark, 'slide', X, 11, 10, 23)
stage.move(shark, 'heave', Y, 2.05, 10, 23)
stage.box('fishbox', 2, -4, 1.2, 1, 0.8, 0.8, 'crate')
stage.box('wheelhouse', -1.1, -4.8, 2.7, 2, 1.8, 1.8, 'vent', parent=hull)

stage = Stage('express-lifts', 'Express Lifts', 'elevators', 'The cars go out of sync. The service car bucks, then drops.')
for x, direction, delay in [(-3.5, Y, 13), (3.5, DOWN, 16)]:
    lift_car = stage.box('lift-car', x, 0, 6.7, 8)
    stage.move(lift_car, 'heave', direction, 1.5, 8, delay)
    stage.move(lift_car, 'heave', direction, 0.38, 1.7, 35)
    if x > 0:
        stage.move(lift_car, 'sink', DOWN, 6, 12, 50)

stage = Stage('freight-ferry', 'Freight Ferry', 'port', 'The barges drift apart and the loading bridge washes away.', -0.67)
west_barge = stage.box('barge', -4.1, 0, 6, 12)
east_barge = stage.box('barge', 4.1, 0, 6, 12)
bridge = stage.box('bridge', 0, 0, 2.2, 3)
stage.move(west_barge, 'slide', Z, 2.6, 14, 15)
stage.move(east_barge, 'slide', Z, -2.6, 14, 15)
stage.move(bridge, 'sink', DOWN, 3, 9, 32)
stage.box('freight', -4, -3, 1.6, 1.2, 1, 1, 'crate')

stage = Stage('tidal-turbine', 'Tidal Turbine', 'turbine', 'The low service decks flood. A turbine arm sweeps the central platform.', -0.57)
stage.box('turbine-base', 0, 0, 9, 8)
west_pontoon = stage.box('pontoon', -5.8, 0, 2.6, 6)
east_pontoon = stage.box('pontoon', 5.8, 0, 2.6, 6)
turbine_arm = stage.box('turbine-arm', 0, 0, 7, 0.48, 0.7, 0.42, 'vent')
stage.move(turbine_arm, 'sweep', Y, 2 * pi, 11, 15)
for i in [west_pontoon, east_pontoon]:
    stage.move(i, 'sink', DOWN, 1.9, 14, 37)

stage = Stage('metro-splice', 'Metro Splice', 'metro', 'The first express takes the center track. A second train enters from the other end.')
stage.box('platform', -3.8, 0, 4.5, 14)
stage.box('platform', 3.8, 0, 4.5, 14)
stage.box('track', 0, 0, 3.1, 15, -0.25)
for z, delay in [(-14, 14), (14, 32)]:
    train = stage.box('train', 0, z, 2.7, 5, 1.6, 1.6, 'vent')
    stage.move(train, 'slide', Z, (-1 if z > 0 else 1) * 23, 12, delay)

stage = Stage('kiln-line', 'Kiln Line', 'kiln', 'The furnace doors open underfoot. Brick carts roll across the firing floor.')
stage.box('brick-bed', -4, 0, 4.2, 11)
stage.box('brick-bed', 4, 0, 4.2, 11)
for z, delay in [(-2.75, 15), (2.75, 32)]:
    furnace_door = stage.box('furnace-door', 0, z, 3.8, 5.5)
    stage.move(furnace_door, 'roll', X, 1.34, 9, delay)
brick_cart = stage.box('brick-cart', -8, 2.4, 2.2, 1.2, 1.2, 1.2, 'vent')
stage.move(brick_cart, 'slide', X, 15, 12, 42)

stage = Stage('parade-floats', 'Parade Floats', 'parade', 'The floats weave out of formation. The brass float starts bouncing.')
for x, z, direction, delay in [(-4, -2, -1, 12), (0, 1, 1, 18), (4, -1, 1, 24)]:
    float = stage.box('float', x, z, 3.9, 8)
    stage.move(float, 'slide', Z, direction * 2.9, 14, delay)
    if x == 0:
        stage.move(float, 'heave', Y, 1.25, 4, 36)

stage = Stage('orbital-dock', 'Orbital Dock', 'space', 'The docking collar rotates. Both shuttle berths retract into space.')
docking_collar = stage.box('docking-collar', 0, 0, 6.5, 6.5)
stage.move(docking_collar, 'sweep', Y, 2 * pi, 29, 16)
for x in [-5.5, 5.5]:
    shuttle = stage.box('shuttle', x, 0, 4.4, 7)
    stage.move(shuttle, 'sink', [1 if x > 0 else -1, 0, 0], 4.5, 20, 34)

stage = Stage('crane-yard', 'Crane Yard', 'crane', 'Suspended loads swing past the work platforms. The outer gantry slews away.')
stage.box('gantry', 0, 0, 10, 5)
service_deck = stage.box('service-deck', -3.6, 4, 2.8, 3)
stage.box('service-deck', 3.6, -4, 2.8, 3)
for x, delay in [(-2, 14), (2, 29)]:
    hook_load = stage.box('hook-load', x, -8, 1.7, 2, 1.9, 1.4, 'vent')
    stage.move(hook_load, 'slide', Z, 13, 10, delay)
stage.move(service_deck, 'roll', Z, 1.05, 16, 41)

stage = Stage('monsoon-dam', 'Monsoon Dam', 'dam', 'Spillway gates sink in sequence. The flood booms sweep across the remaining crossing.', -0.58)
stage.box('gatehouse', -5.6, 0, 3.5, 8)
stage.box('gatehouse', 5.6, 0, 3.5, 8)
for x, delay in [(-2.55, 15), (0, 25), (2.55, 36)]:
    spillgate = stage.box('spillgate', x, 0, 2.5, 4.2)
    stage.move(spillgate, 'sink', DOWN, 2.4, 10, delay)
boom = stage.box('boom', -5.5, -7, 3.5, 0.6, 0.6, 0.6, 'vent')
stage.move(boom, 'slide', Z, 13, 10, 38)

stage = Stage('carnival-wheel', 'Carnival Wheel', 'carnival', 'The gondolas rise and fall at different speeds. The boarding pier gives way.', -0.68)
pier = stage.box('pier', 0, 0, 4.4, 9)
for x, direction in [(-4, Y), (4, DOWN)]:
    gondola = stage.box('gondola', x, 0, 3.5, 6)
    stage.move(gondola, 'heave', direction, 1.9, 9, 14)
    stage.move(gondola, 'roll', Z, 0.22, 4, 32)
stage.move(pier, 'sink', DOWN, 1.8, 19, 44)

stage = Stage('high-wire', 'High Wire', 'circus', 'The walkways sway. The middle span tilts and leaves two narrow landings.')
stage.box('landing', -5, 0, 4, 6)
stage.box('landing', 5, 0, 4, 6)
north_wire_walk = stage.box('wire-walk', 0, -1.65, 6, 1.5)
south_wire_walk = stage.box('wire-walk', 0, 1.65, 6, 1.5)
stage.move(north_wire_walk, 'roll', X, 0.36, 5, 14)
stage.move(south_wire_walk, 'slide', Z, 0.8, 7, 19)
stage.move(north_wire_walk, 'roll', Z, 1.15, 12, 41)

stage = Stage('rolling-stock', 'Rolling Stock', 'rail', 'The freight cars uncouple. A signal gantry sweeps down the consist.')
for z in [-5, 0, 5]:
    railcar = stage.box('railcar', 0, z, 6, 4.95)
    if z:
        stage.move(railcar, 'sink', [0, 0, 1 if z > 0 else -1], 3.4, 17, 18 if z > 0 else 30)
signal_bar = stage.box('signal-bar', 0, -12, 7, 0.7, 1.33, 0.55, 'vent')
stage.move(signal_bar, 'slide', Z, 21, 14, 37)

stage = Stage('signal-dish', 'Signal Dish', 'radio', 'The antenna slews across the receiver. The reflector pitches toward the edge.')
reflector = stage.box('reflector', 0, 0, 10, 9)
feed_arm = stage.box('feed-arm', 0, 0, 8, 0.6, 0.78, 0.46, 'vent')
stage.move(feed_arm, 'sweep', Y, 2 * pi, 15, 13)
stage.move(reflector, 'roll', X, 0.22, 8, 32)
stage.box('catwalk', 0, -5.7, 3, 2.4)

stage = Stage('foundry-press', 'Foundry Press', 'press', 'The press heads descend in sequence. Their transfer tables slide apart.')
stage.box('press-bed', 0, 0, 5, 10)
for x, delay in [(-4.5, 12), (4.5, 25)]:
    transfer_table = stage.box('transfer-table', x, 0, 4, 8)
    stage.move(transfer_table, 'slide', Z, 2.8, 14, 37)
    press_head = stage.box('press-head', x, 0, 3.5, 3, 4.8, 1, 'vent')
    stage.move(press_head, 'heave', DOWN, 3.55, 8, delay)

stage = Stage('orchard-mill', 'Orchard Mill', 'orchard', 'The paddle turns across the mill bridge as the cider vats begin to tip.', -0.67)
stage.box('mill-bridge', 0, 0, 5, 11)
for x in [-4.3, 4.3]:
    cider_vat = stage.box('cider-vat', x, 0, 3.6, 5)
    stage.move(cider_vat, 'roll', Z, 0.47, 7, 29 if x < 0 else 36)
paddle = stage.box('paddle', 0, 0, 4.6, 0.5, 1, 0.8, 'vent')
stage.move(paddle, 'sweep', Y, 2 * pi, 8, 15)

stage = Stage('summit-rescue', 'Summit Rescue', 'alpine', 'The rescue tram leaves the ridge. Snow shelves crumble one by one.')
stage.box('ridge', 0, 0, 5, 8)
tram = stage.box('tram', 4.3, 0, 3.6, 5)
stage.move(tram, 'slide', Z, 5.2, 15, 12)
for z, delay in [(-2.35, 28), (2.35, 42)]:
    snow_shelf = stage.box('snow-shelf', -4.1, z, 3.2, 4.6)
    stage.move(snow_shelf, 'sink', DOWN, 8, 9, delay)

stage = Stage('drum-machine', 'Drum Machine', 'music', 'The drum pads kick on alternating beats. The turntable spins up under the band.')
turntable = stage.box('turntable', 0, 0, 5.7, 6)
stage.move(turntable, 'sweep', Y, 2 * pi, 18, 30)
for x, z, delay in [(-4.5, -2, 12), (-4.5, 2, 15), (4.5, -2, 18), (4.5, 2, 21)]:
    drum_pad = stage.box('drum-pad', x, z, 3.25, 3.8)
    stage.move(drum_pad, 'heave', Y, 0.95, 2.6, delay)

if __name__ == '__main__':
    manifests = [s.manifest() for s in stages]
    output = ROOT / 'assets/maps/tour.json'
    output.write_text('[\n' + ',\n'.join(json.dumps(m,separators=(',',':')) for m in manifests) + '\n]\n')
