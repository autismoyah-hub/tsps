"""objlib.py: object (scenery, "loc") definitions of the revision-239 cache, read and written.
Port of client/rust-client/src/object.rs decode_local (archive 2, group 6, file = id).

  decode(data)          -> dict (every field the client keeps; 'ops' = the stored opcode order)
  encode(d)             -> bytes for one of OURS (the fields an animated hazard or arena piece needs; read back by decode)
  objects()             -> {id: raw bytes}
  shape_for(d)          the shape a harness should place it with (10 if the definition takes it, else its first listed shape)
  seq_cycles(seq)       cycles of one pass of a sequence dict (animlib.decode_sequence), classic or skeletal
"""
import sys, pathlib
sys.dont_write_bytecode = True; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from fxlib import *


class R:
    def __init__(self, b): self.b, self.p = bytes(b), 0
    def u8(self): v = self.b[self.p]; self.p += 1; return v
    def i8(self): v = self.u8(); return v - 256 if v > 127 else v
    def u16(self): v = int.from_bytes(self.b[self.p:self.p + 2], 'big'); self.p += 2; return v
    def u24(self): v = int.from_bytes(self.b[self.p:self.p + 3], 'big'); self.p += 3; return v
    def u32(self): v = int.from_bytes(self.b[self.p:self.p + 4], 'big'); self.p += 4; return v
    def string(self): e = self.b.index(0, self.p); s = self.b[self.p:e]; self.p = e + 1; return s.decode('cp1252')


def decode(data):
    r = R(data)
    d = dict(name='null', models=[], shapes=None, size=[1, 1], interact=2, blocks_projectiles=True, interactive=None, contour=-1, merge_normals=False,
             anim=None, ambient=0, contrast=0, ops={}, recolor=[], retexture=[], mirror=False, shadow=True, scale=[128, 128, 128], offset=[0, 0, 0],
             solid=False, transforms=None, varbit=None, varp=None, randomstart=True, flag90=False, sound=None, opcodes=[])
    opt = lambda: (lambda v: None if v == 65535 else v)(r.u16())
    while True:
        op = r.u8()
        if op == 0: break
        d['opcodes'].append(op)
        if op in (1, 5):
            n = r.u8(); d['models'] = []; d['shapes'] = [] if op == 1 else None
            for _ in range(n):
                d['models'].append(r.u16())
                if op == 1: d['shapes'].append(r.u8())
        elif op in (6, 7):
            n = r.u8(); d['models'] = []; d['shapes'] = [] if op == 6 else None
            for _ in range(n):
                d['models'].append(r.u32())
                if op == 6: d['shapes'].append(r.u8())
        elif op == 2: d['name'] = r.string()
        elif op == 3: r.string()
        elif op == 14: d['size'][0] = r.u8()
        elif op == 15: d['size'][1] = r.u8()
        elif op == 17: d['interact'] = 0; d['blocks_projectiles'] = False
        elif op == 18: d['blocks_projectiles'] = False
        elif op == 19: d['interactive'] = r.u8()
        elif op == 21: d['contour'] = 0
        elif op == 22: d['merge_normals'] = True
        elif op == 23: pass
        elif op == 24: d['anim'] = opt()
        elif op == 27: d['interact'] = 1
        elif op == 28: r.u8()
        elif op == 29: d['ambient'] = r.i8()
        elif op == 39: d['contrast'] = r.i8()          # as stored; the client multiplies by 5
        elif 30 <= op <= 34: d['ops'][op - 30] = r.string()
        elif op == 100: i = r.u8(); r.u8(); d['ops'][i] = r.string()
        elif op in (101, 102):
            i = r.u8(); r.u16(); r.u16()
            if op == 102: r.u16()
            r.u32(); r.u32(); d['ops'][i] = r.string()
        elif op in (40, 41):
            pairs = [(r.u16(), r.u16()) for _ in range(r.u8())]
            d['recolor' if op == 40 else 'retexture'] = pairs
        elif op == 61: r.u16()
        elif op == 62: d['mirror'] = True
        elif op == 64: d['shadow'] = False
        elif 65 <= op <= 67: d['scale'][op - 65] = r.u16()
        elif op == 68: r.u16()
        elif op == 69: r.u8()
        elif 70 <= op <= 72: v = r.u16(); d['offset'][op - 70] = v - 65536 if v > 32767 else v
        elif op == 73: pass
        elif op == 74: d['solid'] = True
        elif op == 75: r.u8()
        elif op in (77, 92):
            d['varbit'] = opt(); d['varp'] = opt()
            fb = opt() if op == 92 else None
            ids = [opt() for _ in range(r.u8() + 1)]
            d['transforms'] = ids + [fb]
        elif op == 78: d['sound'] = r.u16(); r.u8(); r.u8()
        elif op == 79:
            r.u16(); r.u16(); r.u8(); r.u8()
            for _ in range(r.u8()): r.u16()
        elif op == 81: d['contour'] = r.u8() * 256
        elif op == 82: r.u16()
        elif op == 89: d['randomstart'] = False
        elif op == 90: d['flag90'] = True
        elif op == 91: r.u8()
        elif op == 93: r.u8(); r.u16(); r.u8(); r.u16()
        elif op == 95: r.u8()
        elif op == 96: r.u8()
        elif op == 200: r.u16()
        elif op == 249:
            for _ in range(r.u8()):
                k = r.u8(); r.u24()
                if k == 1: r.string()
                elif k == 2: r.p += 8
                else: r.u32()
    assert r.p == len(r.b), 'object definition trailing bytes'
    return d


def encode(d):
    """One of ours. Fields: name, models [ids], shapes [per model] or None (None = shape 10 only), size [x, y], anim (sequence id),
    randomstart (False writes opcode 89: every instance starts at frame 0), ambient, contrast (stored), recolor, scale [x, y, z],
    solid (True = blocks movement; False writes opcode 17: walk-through, as a pool or a floor mark must be), blocks_projectiles,
    interactive (0 / 1), ops {index: text}, shadow (False writes opcode 64), mirror."""
    b = bytearray()
    shapes = d.get('shapes')
    b += bytes([6 if shapes else 7, len(d['models'])])
    for i, m in enumerate(d['models']):
        b += int(m).to_bytes(4, 'big')
        if shapes: b += bytes([shapes[i]])
    b += bytes([2]) + d.get('name', 'null').encode('cp1252') + b'\0'
    sx, sz = d.get('size', [1, 1])
    if sx != 1: b += bytes([14, sx])
    if sz != 1: b += bytes([15, sz])
    if not d.get('solid', False): b += bytes([17])
    elif not d.get('blocks_projectiles', True): b += bytes([18])
    if d.get('interactive') is not None: b += bytes([19, d['interactive']])
    if d.get('anim') is not None: b += bytes([24]) + int(d['anim']).to_bytes(2, 'big')
    if d.get('ambient', 0): b += bytes([29, d['ambient'] & 255])
    if d.get('contrast', 0): b += bytes([39, d['contrast'] & 255])
    for i, t in sorted((d.get('ops') or {}).items()):
        b += bytes([30 + int(i)]) + t.encode('cp1252') + b'\0'
    if d.get('recolor'):
        b += bytes([40, len(d['recolor'])])
        for a, c in d['recolor']: b += int(a).to_bytes(2, 'big') + int(c).to_bytes(2, 'big')
    if d.get('mirror'): b += bytes([62])
    if not d.get('shadow', True): b += bytes([64])
    for i, v in enumerate(d.get('scale', [128, 128, 128])):
        if v != 128: b += bytes([65 + i]) + int(v).to_bytes(2, 'big')
    if not d.get('randomstart', True): b += bytes([89])
    b += b'\0'
    back = decode(bytes(b))
    for k in ('name', 'models', 'anim', 'ambient', 'contrast', 'randomstart', 'mirror', 'shadow'):
        if k in d: assert back[k] == d[k], (k, back[k], d[k])
    assert back['interact'] == (2 if d.get('solid', False) else 0)
    return bytes(b)


_objs = None


def objects():
    global _objs
    if _objs is None: _objs = al.cache().files(2, 6)
    return _objs


def obj(oid): return decode(objects()[oid])


def shape_for(d):
    if d['shapes'] is None: return 10
    for s in (10, 11, 22, 4, 0, 9):
        if s in d['shapes']: return s
    return d['shapes'][0]


def models_for(d, shape):
    if d['shapes'] is None: return list(d['models']) if shape == 10 else []
    return [m for m, s in zip(d['models'], d['shapes']) if s == shape][:1]


def seq_cycles(s):
    if s['cached_animation'] is not None: return s['cached_range'][1] - s['cached_range'][0]
    return sum(s['lengths'])


def seq_kind(s):
    if s['cached_animation'] is not None: return 'skeletal'
    return 'classic' if s['frames'] else 'empty'
