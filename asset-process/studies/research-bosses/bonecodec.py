"""Decoders and ENCODERS for the three files a skeletal (bone-weight, keyframe) creature needs, revision 239.

    model:      decode_model(bytes) -> dict        encode_model(dict, hints=True) -> bytes
    frame base: decode_base(bytes)  -> dict        encode_base(dict) -> bytes       (classic groups + bone section)
    keyframes:  decode_anim(bytes)  -> dict        encode_anim(dict) -> bytes       (archive 22 file)

Each decoder follows the client's own (client/rust-client/src/model.rs, animation.rs Skeleton::decode,
cached_animation.rs, curve.rs) field for field; SKELETAL.md describes the layouts. Nothing here touches a cache.

Model dict (all lists; coordinates as stored: x right, y DOWN, z):
  format            'legacy' | 'extended' | 'skinned_legacy' | 'skinned'   (trailer none | ff ff | ff fe | ff fd)
  vertices          [(x, y, z)]
  vertex_skins      [label] or None            bone_weights  [[(bone, weight 0..255), ...] per vertex] or None
  faces             [(a, b, c)]                colors        [u16]
  render_types, priorities, alpha, face_skins, depth (signed per-face byte), face_textures (-1 = none), texture_coords:
                    per-face lists or None;  priority = the uniform priority when priorities is None
  texture_types, texture_triangles, complex_raw (bytes of the complex mapping sections, carried), metadata
  HINTS (only so that a decoded real file re-encodes to the same BYTES; an author never supplies them):
  _vflags (per-vertex delta flags as stored), _ops (per-face strip opcode 1..4), _depth_marker and _meta_marker (the
  stored marker bytes: any non-zero metadata marker means "present"; real files use 1 and 5)
encode_model writes the 'skinned' layout (trailer ff fd) whatever the source was, or 'extended' when asked and no
weights are present. With hints=False it chooses flags and strip opcodes itself (flags = delta is not zero; strip
opcode 2, 3 or 4 when two indices carry over, else 1).

usage as a tool: bonecodec.py model FILE | base FILE | anim FILE    prints a summary
"""
import struct, sys, pathlib

TRAILER = {b'\xff\xfd': 'skinned', b'\xff\xfe': 'skinned_legacy', b'\xff\xff': 'extended'}


class Rd:
    def __init__(self, d, p=0, end=None): self.d, self.p, self.end = d, p, len(d) if end is None else end
    def u8(self):
        assert self.p < self.end, 'truncated'; v = self.d[self.p]; self.p += 1; return v
    def i8(self):
        v = self.u8(); return v - 256 if v > 127 else v
    def u16(self):
        assert self.p + 2 <= self.end, 'truncated'; v = (self.d[self.p] << 8) | self.d[self.p + 1]; self.p += 2; return v
    def u32(self):
        assert self.p + 4 <= self.end, 'truncated'; v = int.from_bytes(self.d[self.p:self.p + 4], 'big'); self.p += 4; return v
    def f32(self):
        assert self.p + 4 <= self.end, 'truncated'; v = struct.unpack_from('>f', self.d, self.p)[0]; self.p += 4; return v
    def take(self, n):
        assert self.p + n <= self.end, 'truncated'; v = bytes(self.d[self.p:self.p + n]); self.p += n; return v
    def ssmart(self):
        if self.d[self.p] < 128: return self.u8() - 64
        return self.u16() - 49152
    def left(self): return self.end - self.p


def ssmart(v):
    if -64 <= v < 64: return bytes([v + 64])
    if -16384 <= v < 16384: return struct.pack('>H', v + 49152)
    raise ValueError(f'value {v} outside the signed-smart range -16384..16383')


# ------------------------------------------------------------------------------------------------------- model
def decode_model(data):
    tr = bytes(data[-2:]); fmt = TRAILER.get(tr, 'legacy')
    extended = fmt in ('extended', 'skinned'); skinned = fmt in ('skinned', 'skinned_legacy')
    fsize = 18 if fmt == 'legacy' else 26 if fmt == 'skinned' else 23
    end = len(data) - fsize; f = Rd(data, end)
    nv, nf, nt = f.u16(), f.u16(), f.u8()
    render = f.u8(); priority = f.u8(); alpha = f.u8(); fskin = f.u8()
    ftex = f.u8() if extended else render
    vskin = f.u8(); bones = f.u8() if skinned else 0
    xlen, ylen, zlen, ilen = f.u16(), f.u16(), f.u16(), f.u16()
    tlen = f.u16() if extended else 0
    vlen = f.u16() if skinned else (nv if vskin else 0)
    b = Rd(data, 0, end)
    ttypes = list(b.take(nt)) if extended else [0] * nt
    simple = sum(1 for t in ttypes if t == 0); cx = nt - simple; cube = sum(1 for t in ttypes if t == 2)
    vflags = b.take(nv)
    if extended:
        s_render = b.take(nf if render else 0); s_ops = b.take(nf); s_prio = b.take(nf if priority == 255 else 0)
        s_fskin = b.take(nf if fskin else 0); s_vdata = b.take(vlen); s_alpha = b.take(nf if alpha else 0)
        s_idx = b.take(ilen); s_tex = b.take(nf * 2 if ftex else 0); s_coord = b.take(tlen); s_col = b.take(nf * 2)
        s_x = b.take(xlen); s_y = b.take(ylen); s_z = b.take(zlen); s_tri = b.take(simple * 6)
        complex_raw = b.take(cx * 6 + cx * 6 + cx * 2 + cx + cx * 2 + cube * 2)
    else:
        s_ops = b.take(nf); s_prio = b.take(nf if priority == 255 else 0); s_fskin = b.take(nf if fskin else 0)
        s_render = b.take(nf if render else 0); s_vdata = b.take(vlen); s_alpha = b.take(nf if alpha else 0)
        s_idx = b.take(ilen); s_col = b.take(nf * 2); s_tri = b.take(nt * 6)
        s_x = b.take(xlen); s_y = b.take(ylen); s_z = b.take(zlen); s_tex = s_coord = b''; complex_raw = b''
    rx, ry, rz = Rd(s_x), Rd(s_y), Rd(s_z); p = [0, 0, 0]; V = []
    for fl in vflags:
        assert fl & ~7 == 0, 'vertex flags'
        if fl & 1: p[0] += rx.ssmart()
        if fl & 2: p[1] += ry.ssmart()
        if fl & 4: p[2] += rz.ssmart()
        V.append(tuple(p))
    assert rx.left() == ry.left() == rz.left() == 0, 'coordinate sections'
    rv = Rd(s_vdata)
    skins = list(rv.take(nv)) if vskin else None
    weights = None
    if bones:
        weights = []
        for _ in range(nv):
            n = rv.u8(); weights.append([(rv.u8(), rv.u8()) for _ in range(n)])
    assert rv.left() == 0, 'vertex data section'
    ri = Rd(s_idx); a = bb = c = last = 0; F = []
    for op in s_ops:
        if op == 1: a = last + ri.ssmart(); bb = a + ri.ssmart(); c = bb + ri.ssmart()
        elif op == 2: bb = c; c = last + ri.ssmart()
        elif op == 3: a = c; c = last + ri.ssmart()
        elif op == 4: a, bb = bb, a; c = last + ri.ssmart()
        else: raise ValueError('triangle opcode')
        last = c; F.append((a, bb, c))
    assert ri.left() == 0, 'index section'
    colors = list(struct.unpack(f'>{nf}H', s_col))
    m = dict(format=fmt, vertices=V, vertex_skins=skins, bone_weights=weights, faces=F, colors=colors,
             priorities=list(s_prio) if priority == 255 else None, priority=0 if priority == 255 else priority,
             alpha=list(s_alpha) if alpha else None, face_skins=list(s_fskin) if fskin else None,
             texture_types=ttypes, complex_raw=complex_raw, _vflags=list(vflags), _ops=list(s_ops))
    m['texture_triangles'] = [struct.unpack_from('>HHH', s_tri, 6 * i) for i in range(simple if extended else nt)]
    if extended:
        m['render_types'] = list(s_render) if render else None
        m['face_textures'] = [((v - 1) & 0xffff) - (65536 if ((v - 1) & 0xffff) > 32767 else 0) for v in struct.unpack(f'>{nf}H', s_tex)] if ftex else None
        if ftex and nt > 0:
            rc = Rd(s_coord); m['texture_coords'] = [(rc.u8() - 1) if t != -1 else 0 for t in m['face_textures']]
            assert rc.left() == 0, 'texture coordinate section'
        else:
            assert tlen == 0; m['texture_coords'] = None
        m['metadata'] = None
        mk = b.u8(); m['_meta_marker'] = mk
        if mk != 0: m['metadata'] = ((b.u16(), b.u16(), b.u16()), b.u32())
    else:
        # legacy: the render byte carries the flat bit and the texture mapping; kept raw
        m['render_types'] = list(s_render) if render else None
        m['face_textures'] = None; m['texture_coords'] = None; m['metadata'] = None
    m['depth'] = None; m['_depth_marker'] = None
    if skinned:
        mk = b.u8(); m['_depth_marker'] = mk
        if mk == 1: m['depth'] = [v - 256 if v > 127 else v for v in b.take(nf)]
    assert b.left() == 0, f'{b.left()} body bytes unread'
    return m


def choose_ops(faces):
    """Strip opcodes for a face list, as the decoder will replay them (a, b, c registers; `last` = previous c)."""
    ops = []; a = b = c = None
    for (x, y, z) in faces:
        if a is None: op = 1
        elif x == a and y == c: op = 2          # b = c
        elif x == c and y == b: op = 3          # a = c
        elif x == b and y == a: op = 4          # swap a, b
        else: op = 1
        ops.append(op); a, b, c = x, y, z
    return ops


def encode_model(m, hints=True, fmt='skinned'):
    assert fmt in ('skinned', 'extended')
    V, F = m['vertices'], m['faces']; nv, nf = len(V), len(F)
    assert 0 < nv <= 65535 and 0 < nf <= 65535, 'model count limit'
    weights = m.get('bone_weights'); skins = m.get('vertex_skins')
    assert fmt == 'skinned' or not weights, 'bone weights need the skinned layout'
    tt = list(m.get('texture_types') or []); tri = [t for t in (m.get('texture_triangles') or [])]
    simple = sum(1 for t in tt if t == 0); cx = len(tt) - simple; cube = sum(1 for t in tt if t == 2)
    complex_raw = bytes(m.get('complex_raw') or b'')
    assert len(complex_raw) == cx * 17 + cube * 2, 'complex texture mapping bytes'
    assert len(tri) == simple, 'one texture triangle per simple mapping'
    vf = m.get('_vflags') if hints else None
    flags = bytearray(); xyz = [bytearray(), bytearray(), bytearray()]; prev = [0, 0, 0]
    for i, row in enumerate(V):
        row = [int(v) for v in row]; fl = 0
        for ax in range(3):
            d = row[ax] - prev[ax]
            if d or (vf is not None and vf[i] & (1 << ax)): fl |= 1 << ax; xyz[ax] += ssmart(d)
        flags.append(fl); prev = row
    ops = (m.get('_ops') if hints else None) or choose_ops(F)
    idx = bytearray(); a = b = c = last = 0
    for op, (x, y, z) in zip(ops, F):
        assert all(0 <= k < nv for k in (x, y, z)), 'triangle bounds'
        if op == 1: idx += ssmart(x - last) + ssmart(y - x) + ssmart(z - y)
        else:
            want = {2: (a, c), 3: (c, b), 4: (b, a)}[op]
            assert (x, y) == want, 'strip opcode does not fit this face'
            idx += ssmart(z - last)
        a, b, c = x, y, z; last = z
    def per_face(key, signed=False):
        v = m.get(key)
        if v is None: return b''
        assert len(v) == nf, key + ' count'
        return bytes((x & 255) for x in v)
    render = per_face('render_types'); prio = per_face('priorities'); alpha = per_face('alpha'); fskin = per_face('face_skins')
    vdata = bytearray()
    if skins is not None:
        assert len(skins) == nv; vdata += bytes(skins)
    if weights is not None:
        assert len(weights) == nv
        for w in weights:
            assert len(w) <= 255; vdata.append(len(w))
            for bone, wt in w:
                assert 0 <= bone <= 255 and 0 <= wt <= 255, 'bone index and weight are one byte each'; vdata += bytes((bone, wt))
    ftex = m.get('face_textures'); coords = m.get('texture_coords'); tex = b''; coord = bytearray()
    if ftex is not None:
        assert len(ftex) == nf; tex = b''.join(struct.pack('>H', (t + 1) & 0xffff) for t in ftex)
        if tt:
            for i, t in enumerate(ftex):
                if t != -1: coord.append(((coords[i] if coords is not None else -1) + 1) & 255)
    col = struct.pack(f'>{nf}H', *m['colors'])
    lens = [len(xyz[0]), len(xyz[1]), len(xyz[2]), len(idx), len(coord), len(vdata)]
    assert all(v <= 65535 for v in lens), 'a model section is over 65535 bytes'
    md = m.get('metadata'); meta = b'\0' if md is None else bytes([(m.get('_meta_marker') if hints else 0) or 1]) + struct.pack('>HHHI', *md[0], md[1])
    body = bytes(tt) + bytes(flags) + render + bytes(ops) + prio + fskin + bytes(vdata) + alpha + bytes(idx) + tex + bytes(coord) + col \
        + bytes(xyz[0]) + bytes(xyz[1]) + bytes(xyz[2]) + b''.join(struct.pack('>HHH', *t) for t in tri) + complex_raw + meta
    depth = m.get('depth')
    if fmt == 'skinned':
        if depth is not None:
            assert len(depth) == nf; body += b'\1' + bytes((d & 255) for d in depth)
        else:
            mk = m.get('_depth_marker') if hints else 0
            body += bytes([mk if mk not in (None, 1) else 0])
        footer = struct.pack('>HHBBBBBBBBHHHHHH', nv, nf, len(tt), bool(render), 255 if m.get('priorities') is not None else m.get('priority', 0),
                             m.get('alpha') is not None, m.get('face_skins') is not None, ftex is not None, skins is not None, weights is not None, *lens) + b'\xff\xfd'
    else:
        assert depth is None, 'depth offsets need the skinned layout'
        assert len(vdata) == (nv if skins is not None else 0)
        footer = struct.pack('>HHBBBBBBBHHHHH', nv, nf, len(tt), bool(render), 255 if m.get('priorities') is not None else m.get('priority', 0),
                             m.get('alpha') is not None, m.get('face_skins') is not None, ftex is not None, skins is not None, *lens[:5]) + b'\xff\xff'
    return body + footer


def model_equal(a, b):
    """True when two decoded models hold the same content (hints ignored)."""
    keys = ('vertices', 'vertex_skins', 'bone_weights', 'faces', 'colors', 'render_types', 'priorities', 'priority', 'alpha', 'face_skins',
            'depth', 'face_textures', 'texture_coords', 'texture_types', 'texture_triangles', 'complex_raw', 'metadata')
    norm = lambda v: [tuple(tuple(y) if isinstance(y, (list, tuple)) else y for y in x) if isinstance(x, (list, tuple)) else x for x in v] if isinstance(v, (list, tuple)) else v
    return [k for k in keys if norm(a.get(k)) != norm(b.get(k))]


# -------------------------------------------------------------------------------------------------- frame base
def decode_base(data):
    """{'types', 'labels', 'bones': [{'parent', 'matrices': [16 floats per pose], 'extra': [3 floats per pose]}], 'poses',
    '_tail': how the stored file ends when it has no bones (b'' or the two zero bytes)}. Floats are kept as their 32-bit
    patterns' exact values (Python floats hold every f32 exactly), so encode_base returns the same bytes."""
    r = Rd(data); n = r.u8(); types = list(r.take(n)); sizes = list(r.take(n)); labels = [list(r.take(k)) for k in sizes]
    bones = []; poses = 0; tail = b''
    if r.left() > 0:
        p0 = r.p; count = r.u16()
        if count > 0:
            poses = r.u8()
            for _ in range(count):
                parent = r.u16(); parent = parent - 65536 if parent > 32767 else parent
                ms, ex = [], []
                for _ in range(poses):
                    ms.append([r.f32() for _ in range(16)]); ex.append([r.f32() for _ in range(3)])
                bones.append(dict(parent=parent, matrices=ms, extra=ex))
        else:
            tail = bytes(data[p0:])
            r.p = len(data)
    assert r.left() == 0, 'base trailing bytes'
    return dict(types=types, labels=labels, bones=bones, poses=poses, _tail=tail)


def encode_base(b):
    out = bytearray([len(b['types'])]) + bytes(b['types']) + bytes(len(l) for l in b['labels'])
    assert len(b['types']) == len(b['labels']) <= 255
    for l in b['labels']: out += bytes(l)
    bones = b.get('bones') or []
    if not bones:
        return bytes(out + (b.get('_tail', b'\0\0')))
    poses = b.get('poses') or len(bones[0]['matrices'])
    assert len(bones) <= 65535 and 1 <= poses <= 255
    out += struct.pack('>HB', len(bones), poses)
    for i, bone in enumerate(bones):
        assert bone['parent'] < len(bones), 'bone parent index'
        assert len(bone['matrices']) == poses and len(bone['extra']) == poses
        out += struct.pack('>H', bone['parent'] & 0xffff)
        for m, e in zip(bone['matrices'], bone['extra']):
            out += struct.pack('>16f', *m) + struct.pack('>3f', *e)
    return bytes(out)


# ---------------------------------------------------------------------------------------------- keyframe file
CHANNEL_NAMES = {1: 'rot.x', 2: 'rot.y', 3: 'rot.z', 4: 'tran.x', 5: 'tran.y', 6: 'tran.z', 7: 'scale.x', 8: 'scale.y', 9: 'scale.z',
                 10: 'c10', 11: 'c11', 12: 'c12', 13: 'c13', 14: 'c14', 15: 'c15', 16: 'alpha'}


def decode_anim(data):
    """{'version', 'base', 'range': [a, b], 'pose', 'channels': [{'kind', 'target', 'channel', 'curve'}]}
    curve = {'byte0', 'before', 'after', 'weighted', 'keys': [[time, value, in_x, in_y, out_x, out_y], ...]}
    Floats pass through Python doubles, which hold every 32-bit float exactly (all 914 real files re-encode to the
    same bytes)."""
    r = Rd(data); out = dict(version=r.u8(), base=r.u16(), range=[r.u16(), r.u16()], pose=r.u8(), channels=[])
    for _ in range(r.u16()):
        kind = r.u8(); target = r.ssmart(); ch = r.u8()
        n = r.u16(); byte0 = r.u8(); before = r.u8(); after = r.u8(); weighted = r.u8()
        keys = []
        for _ in range(n):
            t = r.u16(); t = t - 65536 if t > 32767 else t
            keys.append([t] + list(struct.unpack_from('>5f', data, r.p))); r.p += 20
        out['channels'].append(dict(kind=kind, target=target, channel=ch, curve=dict(byte0=byte0, before=before, after=after, weighted=weighted, keys=keys)))
    assert r.left() == 0, 'animation trailing bytes'
    return out


def encode_anim(a):
    out = bytearray(struct.pack('>BHHHBH', a['version'], a['base'], a['range'][0], a['range'][1], a['pose'], len(a['channels'])))
    for c in a['channels']:
        cv = c['curve']; keys = cv['keys']
        assert 1 <= len(keys) <= 65535, 'a curve needs at least one key'
        assert all(keys[i][0] <= keys[i + 1][0] for i in range(len(keys) - 1)), 'key times must not decrease'
        assert cv['before'] <= 4 and cv['after'] <= 4
        out += bytes([c['kind']]) + ssmart(c['target']) + bytes([c['channel']])
        out += struct.pack('>HBBBB', len(keys), cv.get('byte0', 0), cv['before'], cv['after'], int(cv['weighted']))
        for k in keys: out += struct.pack('>H5f', k[0] & 0xffff, *k[1:6])
    return bytes(out)


def _main():
    kind, path = sys.argv[1], pathlib.Path(sys.argv[2]); d = path.read_bytes()
    if kind == 'model':
        m = decode_model(d); w = m['bone_weights']
        print(f"{path.name}: {m['format']} {len(m['vertices'])} vertices {len(m['faces'])} faces; labels {m['vertex_skins'] is not None}; weights {w is not None}")
        if w:
            import collections
            print('  influences per vertex', dict(sorted(collections.Counter(len(x) for x in w).items())), ' bones used', len({b for x in w for b, _ in x}))
        print('  re-encode with hints identical:', encode_model(m) == d, '| without hints identical:', encode_model(m, hints=False) == d)
    elif kind == 'base':
        b = decode_base(d); print(f"{path.name}: {len(b['types'])} groups, {len(b['bones'])} bones, {b['poses']} poses; identical: {encode_base(b) == d}")
        for i, bone in enumerate(b['bones']): print(f"  bone {i} parent {bone['parent']} t {[round(v, 2) for v in bone['matrices'][0][12:15]]} extra {[round(v, 2) for v in bone['extra'][0]]}")
    else:
        a = decode_anim(d); print(f"{path.name}: version {a['version']} base {a['base']} range {a['range']} pose {a['pose']} channels {len(a['channels'])}; identical: {encode_anim(a) == d}")


if __name__ == '__main__':
    _main()
