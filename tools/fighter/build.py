"""Rebuild the original skinned fighter with Python's standard library."""
import json
from pathlib import Path
import struct

from model import COLORS, build

ROOT = Path(__file__).resolve().parents[2]
ARRAY_BUFFER = 34962
ELEMENT_ARRAY_BUFFER = 34963
FORMATS = {
    'POSITION': ('f', 'VEC3'),
    'NORMAL': ('f', 'VEC3'),
    'TEXCOORD_0': ('f', 'VEC2'),
    'COLOR_0': ('f', 'VEC3'),
    'JOINTS_0': ('H', 'VEC4'),
    'WEIGHTS_0': ('f', 'VEC4'),
}


def translation(offset):
    return (1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, *offset, 1)


def linear_channel(channel):
    value = channel / 255
    return value / 12.92 if value <= .04045 else ((value + .055) / 1.055) ** 2.4


def export():
    rig = json.loads((ROOT / 'config/character.json').read_text())
    names = [body['part'] for body in rig['bodies']]
    anchors = {body['part']: body['position'] for body in rig['bodies']}
    parents = {}
    for joint in rig['joints']:
        parents[joint['child']] = joint['parent']
        anchors[joint['child']] = joint['anchor']

    nodes = []
    for name in names:
        origin = anchors[parents[name]] if name in parents else (0, 0, 0)
        local = [a - b for a, b in zip(anchors[name], origin)]
        node = {'name': name, 'translation': local}
        children = [i for i, child in enumerate(names) if parents.get(child) == name]
        if children:
            node['children'] = children
        nodes.append(node)
    root = names.index('pelvis')
    nodes.append({'name': 'RoofCrew', 'mesh': 0, 'skin': 0})
    doc = {
        'asset': {'version': '2.0', 'generator': 'gang bbeasts studio bean'},
        'scene': 0, 'scenes': [{'nodes': [root, len(names)]}], 'nodes': nodes,
        'bufferViews': [], 'accessors': [],
    }
    buffer = bytearray()

    def accessor(values, dtype, dimension, target=None, bounds=False):
        buffer.extend(bytes(-len(buffer) % 4))
        start = len(buffer)
        width = {'SCALAR': 1, 'VEC2': 2, 'VEC3': 3, 'VEC4': 4, 'MAT4': 16}[dimension]
        pack = struct.Struct('<' + dtype * width).pack
        for value in values:
            buffer.extend(pack(*value))
        view = {'buffer': 0, 'byteOffset': start, 'byteLength': len(buffer) - start}
        if target:
            view['target'] = target
        entry = {
            'bufferView': len(doc['bufferViews']), 'componentType': {'f': 5126, 'H': 5123}[dtype],
            'count': len(values), 'type': dimension,
        }
        if bounds:
            entry.update(min=list(map(min, zip(*values))), max=list(map(max, zip(*values))))
        doc['bufferViews'].append(view)
        doc['accessors'].append(entry)
        return len(doc['accessors']) - 1

    palette = [tuple(map(linear_channel, bytes.fromhex(color))) for color in COLORS]
    data, indices = build().attributes(palette, names)
    attributes = {
        name: accessor(values, *FORMATS[name], ARRAY_BUFFER, bounds=name == 'POSITION')
        for name, values in data.items()
    }
    index_accessor = accessor([(i,) for i in indices], 'H', 'SCALAR', ELEMENT_ARRAY_BUFFER)
    primitive = {'attributes': attributes, 'indices': index_accessor, 'material': 0}
    doc['meshes'] = [{'name': 'Studio bean', 'primitives': [primitive]}]
    doc['materials'] = [{
        'name': 'Vinyl and wrist ink', 'pbrMetallicRoughness': {'metallicFactor': 0, 'roughnessFactor': .47},
    }]
    inverses = [translation([-v for v in anchors[name]]) for name in names]
    doc['skins'] = [{
        'skeleton': root, 'joints': list(range(len(names))),
        'inverseBindMatrices': accessor(inverses, 'f', 'MAT4'),
    }]
    doc['buffers'] = [{'byteLength': len(buffer)}]
    metadata = json.dumps(doc, separators=(',', ':')).encode()
    metadata += b' ' * (-len(metadata) % 4)
    buffer.extend(bytes(-len(buffer) % 4))
    length = 28 + len(metadata) + len(buffer)
    header = struct.pack('<4sIIII', b'glTF', 2, length, len(metadata), 0x4e4f534a)
    binary_header = struct.pack('<I4s', len(buffer), b'BIN\0')
    (ROOT / 'assets/fighters/roof-crew.glb').write_bytes(header + metadata + binary_header + buffer)


if __name__ == '__main__':
    export()
