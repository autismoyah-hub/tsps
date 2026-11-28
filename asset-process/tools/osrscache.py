"""Read-only reader for the revision-239 game cache and its model format.

Private tooling: output derived from the cache stays under <ROOT>.
Model decode is a line-for-line port of rust-client/src/model.rs (osrs-workspace f745d18).
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import bz2, zlib, colorsys
from pathlib import Path

CACHE = Path(_AP.fix(r'<OSRS_CACHE_239>'))


def group(archive, gid, cache=CACHE):
    with (cache / f'main_file_cache.idx{archive}').open('rb') as f:
        f.seek(gid * 6)
        e = f.read(6)
    if len(e) != 6:
        raise KeyError((archive, gid))
    length = int.from_bytes(e[:3], 'big')
    sector = int.from_bytes(e[3:], 'big')
    if not (0 < length < 64 << 20) or not sector:
        raise KeyError((archive, gid))
    out = bytearray()
    chunk = 0
    with (cache / 'main_file_cache.dat2').open('rb') as d:
        while len(out) < length:
            d.seek(sector * 520)
            b = d.read(520)
            h = 10 if gid > 65535 else 8
            assert int.from_bytes(b[:h - 6], 'big') == gid
            assert int.from_bytes(b[h - 6:h - 4], 'big') == chunk
            assert b[h - 1] == archive
            out += b[h:h + min(length - len(out), 520 - h)]
            sector = int.from_bytes(b[h - 4:h - 1], 'big')
            chunk += 1
    return bytes(out)


def unpack(raw):
    kind = raw[0]
    n = int.from_bytes(raw[1:5], 'big')
    if kind == 0:
        return raw[5:5 + n]
    payload = raw[9:9 + n]
    return bz2.decompress(b'BZh1' + payload) if kind == 1 else zlib.decompress(payload, 31)


class R:
    def __init__(self, data):
        self.d, self.p = data, 0

    def u8(self):
        v = self.d[self.p]; self.p += 1; return v

    def u16(self):
        v = int.from_bytes(self.d[self.p:self.p + 2], 'big'); self.p += 2; return v

    def u32(self):
        v = int.from_bytes(self.d[self.p:self.p + 4], 'big'); self.p += 4; return v

    def bytes(self, n):
        v = self.d[self.p:self.p + n]
        assert len(v) == n, 'truncated'
        self.p += n
        return v

    def ssmart(self):
        if self.d[self.p] < 128:
            return self.u8() - 64
        return self.u16() - 49152

    def left(self):
        return len(self.d) - self.p


def decode_model(data, preserve_layout=False):
    suffix = data[-2:]
    fmt = {b'\xff\xfd': 'skinned-extended', b'\xff\xfe': 'skinned-legacy', b'\xff\xff': 'extended'}.get(suffix, 'legacy')
    extended = fmt in ('extended', 'skinned-extended')
    skinned = fmt in ('skinned-legacy', 'skinned-extended')
    fsize = {'legacy': 18, 'skinned-extended': 26}.get(fmt, 23)
    end = len(data) - fsize
    f = R(data[end:])
    nv, nf, nt = f.u16(), f.u16(), f.u8()
    render = f.u8() == 1
    priority = f.u8()
    alpha = f.u8() == 1
    face_skin = f.u8() == 1
    face_texture = (f.u8() == 1) if extended else render
    vertex_skin = f.u8() == 1
    bones = (f.u8() == 1) if skinned else False
    xlen, ylen, zlen, ilen = f.u16(), f.u16(), f.u16(), f.u16()
    tlen = f.u16() if extended else 0
    vlen = f.u16() if skinned else (nv if vertex_skin else 0)
    body = R(data[:end])
    ttypes = list(body.bytes(nt)) if extended else [0] * nt
    simple = sum(1 for t in ttypes if t == 0)
    complex_ = nt - simple
    cube = sum(1 for t in ttypes if t == 2)
    complex_sections = ['']*5
    S = lambda n: R(body.bytes(n))
    vflags = S(nv)
    if extended:
        render_d = S(nf if render else 0); itypes = S(nf); prio_d = S(nf if priority == 255 else 0)
        fskin_d = S(nf if face_skin else 0); vdata = S(vlen); alpha_d = S(nf if alpha else 0)
        idata = S(ilen); tex_d = S(nf * 2 if face_texture else 0); coord_d = S(tlen); color_d = S(nf * 2)
        xd, yd, zd = S(xlen), S(ylen), S(zlen); tri_d = S(simple * 6)
        complex_sections = [body.bytes(n).hex() for n in (complex_*6,complex_*6,complex_*2,complex_,complex_*2+cube*2)]
    else:
        itypes = S(nf); prio_d = S(nf if priority == 255 else 0); fskin_d = S(nf if face_skin else 0)
        render_d = S(nf if render else 0); vdata = S(vlen); alpha_d = S(nf if alpha else 0)
        idata = S(ilen); color_d = S(nf * 2); tri_d = S(nt * 6)
        xd, yd, zd = S(xlen), S(ylen), S(zlen); tex_d = R(b''); coord_d = R(b'')
    verts = []
    vskins = [] if vertex_skin else None
    px = py = pz = 0
    for _ in range(nv):
        fl = vflags.u8()
        px += xd.ssmart() if fl & 1 else 0
        py += yd.ssmart() if fl & 2 else 0
        pz += zd.ssmart() if fl & 4 else 0
        verts.append((px, py, pz))
        if vertex_skin:
            vskins.append(vdata.u8())
    weights = []
    if bones:
        for _ in range(nv):
            weights.append([(vdata.u8(),vdata.u8()) for _ in range(vdata.u8())])
    colors, rtypes, prios, alphas, fskins, ftex = [], [], [], [], [], []
    coords = []
    for _ in range(nf):
        c = color_d.u16()
        coord = -1
        rt = 0
        tex = -1
        if render:
            fl = render_d.u8()
            if extended:
                rt = fl
            else:
                rt = fl & 1
                if fl & 2:
                    coord = fl >> 2
                    tex = c if c != 65535 else -1
                    c = 127
        colors.append(c); rtypes.append(rt)
        prios.append(prio_d.u8() if priority == 255 else priority)
        alphas.append(alpha_d.u8() if alpha else 0)
        fskins.append(fskin_d.u8() if face_skin else 0)
        if extended and face_texture:
            tex = ((tex_d.u16() - 1) & 0xffff)
            tex = -1 if tex == 0xffff else tex
            if nt > 0 and tex != -1:
                coord = coord_d.u8() - 1
        coords.append(coord)
        ftex.append(tex)
    faces = []
    a = b = c = last = 0
    for _ in range(nf):
        op = itypes.u8()
        if op == 1:
            a = last + idata.ssmart(); b = a + idata.ssmart(); c = b + idata.ssmart()
        elif op == 2:
            b = c; c = last + idata.ssmart()
        elif op == 3:
            a = c; c = last + idata.ssmart()
        elif op == 4:
            a, b = b, a; c = last + idata.ssmart()
        else:
            raise ValueError('triangle opcode')
        last = c
        faces.append((a, b, c))
    # trailing sections: simple texture triangles were already sliced off; metadata, then per-face depth offsets
    depth = [0] * nf
    tail_start = body.p
    try:
        if extended and body.u8() != 0:
            body.bytes(10)
        if skinned and body.u8() == 1:
            depth = [(b - 256 if b > 127 else b) * 2 for b in body.bytes(nf)]
    except (AssertionError, IndexError):
        pass
    triangles = [tuple(tri_d.u16() for _ in range(3)) if t == 0 else (0, 0, 0) for t in ttypes]
    extra = {}
    if preserve_layout:
        # Compression choices and out-of-band fields. Geometry, face materials and
        # labels are always regenerated from the editable Blender mesh.
        remaining = data[tail_start:end]
        meta_size = (11 if remaining[:1] == b'\x01' else 1) if extended else 0
        extra['packing'] = dict(format=fmt, priority=priority, render=render, alpha=alpha,
                texture=face_texture, fskin=face_skin,vskin=vertex_skin,bones=weights if bones else None,
                flags=list(vflags.d), ops=list(itypes.d), metadata=remaining[:meta_size].hex(),
                depth_marker=list(remaining[meta_size:meta_size+1]) if skinned else [])
    return dict(**extra,uniform_priority=priority if priority!=255 else None,has_alpha=alpha,
                complex_sections=complex_sections,bone_weights=weights if bones else None,
                texture_coords=coords, texture_types=ttypes, texture_triangles=triangles, has_face_skin=face_skin,
                has_bone_weights=bones, depth_bias=depth, format=fmt, vertices=verts, faces=faces, colors=colors, render_types=rtypes,
                priorities=prios, alpha=alphas, face_textures=ftex, vertex_skins=vskins, face_skins=fskins,
                has_render=render)


def model(mid):
    return decode_model(unpack(group(7, mid)))


def hsl_split(c):
    return (c >> 10) & 63, (c >> 7) & 7, c & 127


def hsl_to_rgb(c):
    """Client palette colour for a 16-bit HSL value, before lighting (brightness 0.8 curve omitted)."""
    h, s, l = hsl_split(c)
    return colorsys.hls_to_rgb(h / 64 + 1 / 128, l / 128, s / 8 + 1 / 16)


def merge(models):
    """Concatenate several decoded models into one (an item or object is often 2-6 models)."""
    models = list(models)
    out = dict(vertices=[], faces=[], colors=[], render_types=[], priorities=[], alpha=[], face_textures=[])
    out.update(texture_coords=[], texture_types=[], texture_triangles=[], depth_bias=[], face_skins=[],
               uniform_priority=(models[0]['uniform_priority'] if models and
                                 all(m['uniform_priority']==models[0]['uniform_priority'] for m in models) else None),
               has_alpha=any(m['has_alpha'] for m in models),
               complex_sections=[''.join(m['complex_sections'][i] for m in models) for i in range(5)],
               bone_weights=([row for m in models for row in (m['bone_weights'] or [[]]*len(m['vertices']))]
                             if any(m['has_bone_weights'] for m in models) else None),
               vertex_skins=[] if any(m['vertex_skins'] is not None for m in models) else None,
               has_face_skin=any(m['has_face_skin'] for m in models),
               has_bone_weights=any(m['has_bone_weights'] for m in models))
    for m in models:
        o = len(out['vertices'])
        t = len(out['texture_types'])
        out['texture_coords'].extend(i+t if i>=0 else -1 for i in m['texture_coords'])
        out['texture_triangles'].extend(tuple(i+o for i in tri) for tri in m['texture_triangles'])
        out['texture_types'].extend(m['texture_types'])
        out['depth_bias'].extend(m['depth_bias'])
        out['face_skins'].extend(m['face_skins'])
        if out['vertex_skins'] is not None:
            out['vertex_skins'].extend(m['vertex_skins'] or [0]*len(m['vertices']))
        out['vertices'] += m['vertices']
        out['faces'] += [(a + o, b + o, c + o) for a, b, c in m['faces']]
        for k in ('colors', 'render_types', 'priorities', 'alpha', 'face_textures'):
            out[k] += m[k]
    return out


def combine_for_render(parts):
    """Retain face order and weld in first-use order, as model_merge.rs does."""
    out = merge(parts)
    if len(parts) < 2:
        return out
    positions = {}
    vertices, skins, weights, faces, triangles = [], [], [], [], []
    for m in parts:
        def index(i):
            p = tuple(m['vertices'][i])
            if p not in positions:
                positions[p] = len(vertices)
                vertices.append(p)
                skins.append(m['vertex_skins'][i] if m['vertex_skins'] is not None else 0)
                if out['has_bone_weights']:
                    weights.append(m['bone_weights'][i] if m['bone_weights'] is not None else [])
            return positions[p]
        faces.extend(tuple(index(i) for i in f) for f in m['faces'])
        triangles.extend(tuple(index(i) for i in tri) if t==0 else (0,0,0)
                         for t,tri in zip(m['texture_types'],m['texture_triangles']))
    out.update(vertices=vertices,faces=faces,vertex_skins=skins,texture_triangles=triangles,
               bone_weights=weights if out['has_bone_weights'] else None)
    return out
