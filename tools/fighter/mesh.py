"""Ring surfaces for the original Skyline crew suit. Dimensions are in metres."""
import math


class SuitMesh:
    def __init__(self):
        self.points = []
        self.faces = []

    def rings(self, rows, sectors=20):
        """Rows give center, elliptical radii, skin weights and the paint above that ring."""
        start = len(self.points)
        for (x, y, z), (rx, rz), influences, _ in rows:
            for side in range(sectors):
                angle = math.tau * side / sectors
                self.points.append(((x + rx * math.cos(angle), y, z + rz * math.sin(angle)), influences))
        for row in range(len(rows) - 1):
            paint = rows[row][3]
            for side in range(sectors):
                a = start + row * sectors + side
                b = start + row * sectors + (side + 1) % sectors
                shade = paint(side, sectors) if callable(paint) else paint
                self.faces.extend([((a, a + sectors, b), shade), ((b, a + sectors, b + sectors), shade)])
        for row, bottom in [(0, True), (len(rows) - 1, False)]:
            center, _, weights, paint = rows[row]
            tip = len(self.points)
            self.points.append((center, weights))
            for side in range(sectors):
                a = start + row * sectors + side
                b = start + row * sectors + (side + 1) % sectors
                self.faces.append(((tip, a, b) if bottom else (tip, b, a),
                                   paint(side, sectors) if callable(paint) else paint))

    def oval(self, center, radii, bone, paint, sectors=16):
        x, y, z = center
        rx, ry, rz = radii
        rows = []
        for i in range(9):
            angle = math.pi * (i / 8 - .5)
            c = max(.001, math.cos(angle))
            rows.append(((x, y + ry * math.sin(angle), z), (rx * c, rz * c), [(bone, 1)], paint))
        self.rings(rows, sectors)

    def attributes(self, palette, bone_names):
        normals = [[0., 0., 0.] for _ in self.points]
        for face, _ in self.faces:
            a, b, c = [self.points[i][0] for i in face]
            ux, uy, uz = [n - m for n, m in zip(b, a)]
            vx, vy, vz = [n - m for n, m in zip(c, a)]
            normal = (uy*vz - uz*vy, uz*vx - ux*vz, ux*vy - uy*vx)
            for i in face:
                for axis in range(3):
                    normals[i][axis] += normal[axis]
        data = {key: [] for key in ('POSITION', 'NORMAL', 'TEXCOORD_0', 'COLOR_0', 'JOINTS_0', 'WEIGHTS_0')}
        lookup, indices = {}, []
        for face, paint in self.faces:
            for i in face:
                key = (i, paint)
                if key not in lookup:
                    lookup[key] = len(lookup)
                    point, influences = self.points[i]
                    normal = normals[i]
                    length = math.hypot(*normal)
                    data['POSITION'].append(point)
                    data['NORMAL'].append(tuple(n/length for n in normal))
                    data['TEXCOORD_0'].append(((paint + .5)/16, .5))
                    data['COLOR_0'].append(palette[paint])
                    joints = [bone_names.index(name) for name, _ in influences]
                    weights = [w for _, w in influences]
                    data['JOINTS_0'].append((joints + [0]*4)[:4])
                    data['WEIGHTS_0'].append((weights + [0.]*4)[:4])
                indices.append(lookup[key])
        return data, indices
