"""Classic (frame) animation codec for the revision-239 cache: read, decode, ENCODE, and pack.

Everything here is a port of, or an inverse of, the client's own decoders:
  rust-client/src/animation.rs   Sequence::decode, Skeleton::decode (the frame base), Frame::decode
  rust-client/src/deform.rs      Model::animate_classic_mask / transform (how a frame moves labelled vertices)
  rust-client/src/disk_cache.rs  reference tables, containers, group splitting
See FORMAT.md beside this file for the format in words.

Cache layout used:
  archive 2, group 12, file N      sequence definition N
  archive 0, group G, file F       classic frame F of frame group G; a sequence names it as (G << 16) | F
  archive 1, group B, file 0       frame base ("skeleton") B; the first two bytes of every frame file are B
  archive 22                       the newer keyframe animations (not written here)
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import bz2, zlib, gzip, struct, io
from pathlib import Path

CACHE = Path(_AP.fix(r'<OSRS_CACHE_239>'))

KIND = {0: 'origin', 1: 'translate', 2: 'rotate', 3: 'scale', 5: 'alpha'}


# ----------------------------------------------------------------------------------------------- cache reading
class Cache:
    def __init__(self, root=CACHE):
        self.root = Path(root)
        self._ref = {}

    def raw(self, archive, gid):
        """The stored container of a group, with its trailing revision bytes."""
        with (self.root / f'main_file_cache.idx{archive}').open('rb') as f:
            f.seek(gid * 6)
            e = f.read(6)
        if len(e) != 6:
            raise KeyError((archive, gid))
        length = int.from_bytes(e[:3], 'big')
        sector = int.from_bytes(e[3:], 'big')
        if not length or not sector:
            raise KeyError((archive, gid))
        out = bytearray()
        chunk = 0
        with (self.root / 'main_file_cache.dat2').open('rb') as d:
            while len(out) < length:
                d.seek(sector * 520)
                b = d.read(520)
                h = 10 if gid > 65535 else 8
                assert int.from_bytes(b[:h - 6], 'big') == gid, 'sector chain: group id'
                assert int.from_bytes(b[h - 6:h - 4], 'big') == chunk, 'sector chain: chunk'
                assert b[h - 1] == archive, 'sector chain: archive'
                out += b[h:h + min(length - len(out), 520 - h)]
                sector = int.from_bytes(b[h - 4:h - 1], 'big')
                chunk += 1
        return bytes(out)

    def reference(self, archive):
        if archive not in self._ref:
            self._ref[archive] = parse_reference(unpack(self.raw(255, archive)))
        return self._ref[archive]

    def files(self, archive, gid):
        """{file id: bytes} of one group."""
        info = self.reference(archive)['groups'][gid]
        return dict(zip(info['files'], split_files(unpack(self.raw(archive, gid)), len(info['files']))))

    def file(self, archive, gid, fid):
        return self.files(archive, gid)[fid]


def unpack(raw):
    kind = raw[0]
    n = int.from_bytes(raw[1:5], 'big')
    if kind == 0:
        return raw[5:5 + n]
    payload = raw[9:9 + n]
    if kind == 1:
        return bz2.decompress(b'BZh1' + payload)
    return zlib.decompress(payload, 31)


def container_span(raw):
    """Length of the container proper inside a stored group (the rest is the revision trailer)."""
    n = int.from_bytes(raw[1:5], 'big')
    return 5 + n + (0 if raw[0] == 0 else 4)


def pack_container(data, kind=2):
    """A gzip (kind 2) or stored (kind 0) container, as the client's Container::parse accepts."""
    if kind == 0:
        return bytes([0]) + len(data).to_bytes(4, 'big') + data
    buf = io.BytesIO()
    with gzip.GzipFile(fileobj=buf, mode='wb', mtime=0) as g:
        g.write(data)
    z = buf.getvalue()
    return bytes([2]) + len(z).to_bytes(4, 'big') + len(data).to_bytes(4, 'big') + z


def split_files(data, count):
    if count == 1:
        return [data]
    chunks = data[-1]
    pos = len(data) - 1 - chunks * count * 4
    sizes = [[0] * count for _ in range(chunks)]
    for c in range(chunks):
        size = 0
        for f in range(count):
            size += int.from_bytes(data[pos:pos + 4], 'big', signed=True)
            pos += 4
            sizes[c][f] = size
    out = [bytearray() for _ in range(count)]
    pos = 0
    for c in range(chunks):
        for f in range(count):
            out[f] += data[pos:pos + sizes[c][f]]
            pos += sizes[c][f]
    return [bytes(o) for o in out]


def join_files(files):
    """Inverse of split_files: one chunk, delta-coded sizes."""
    if len(files) == 1:
        return files[0]
    out = bytearray()
    for f in files:
        out += f
    prev = 0
    for f in files:
        out += (len(f) - prev).to_bytes(4, 'big', signed=True)
        prev = len(f)
    out.append(1)
    return bytes(out)


def parse_reference(d):
    r = Rd(d)
    protocol = r.u8()
    version = r.u32() if protocol >= 6 else 0
    flags = r.u8()
    big = (lambda: r.bigsmart()) if protocol >= 7 else (lambda: r.u16())
    n = big()
    ids = []
    last = 0
    for _ in range(n):
        last += big()
        ids.append(last)
    groups = {g: {} for g in ids}
    if flags & 1:
        for g in ids: groups[g]['name'] = r.u32()
    for g in ids: groups[g]['crc'] = r.u32()
    if flags & 8:
        for g in ids: groups[g]['ucrc'] = r.u32()
    if flags & 2:
        for g in ids: groups[g]['whirlpool'] = r.take(64)
    if flags & 4:
        for g in ids: groups[g]['sizes'] = (r.u32(), r.u32())
    for g in ids: groups[g]['version'] = r.u32()
    counts = [big() for _ in ids]
    for g, c in zip(ids, counts):
        last = 0
        fl = []
        for _ in range(c):
            last += big()
            fl.append(last)
        groups[g]['files'] = fl
    if flags & 1:
        for g in ids:
            groups[g]['file_names'] = [r.u32() for _ in groups[g]['files']]
    assert r.p == len(d), 'reference table trailing bytes'
    return dict(protocol=protocol, version=version, flags=flags, groups=groups)


def build_reference(ref):
    """Inverse of parse_reference (group ids ascending)."""
    w = Wr()
    protocol, flags = ref['protocol'], ref['flags']
    w.u8(protocol)
    if protocol >= 6: w.u32(ref['version'])
    w.u8(flags)
    big = w.bigsmart if protocol >= 7 else w.u16
    ids = sorted(ref['groups'])
    G = ref['groups']
    big(len(ids))
    last = 0
    for g in ids:
        big(g - last); last = g
    if flags & 1:
        for g in ids: w.u32(G[g].get('name', 0))
    for g in ids: w.u32(G[g]['crc'])
    if flags & 8:
        for g in ids: w.u32(G[g].get('ucrc', 0))
    if flags & 2:
        for g in ids: w.raw(G[g].get('whirlpool', bytes(64)))
    if flags & 4:
        for g in ids:
            a, b = G[g].get('sizes', (0, 0)); w.u32(a); w.u32(b)
    for g in ids: w.u32(G[g]['version'])
    for g in ids: big(len(G[g]['files']))
    for g in ids:
        last = 0
        for f in G[g]['files']:
            big(f - last); last = f
    if flags & 1:
        for g in ids:
            names = G[g].get('file_names') or [0] * len(G[g]['files'])
            for x in names: w.u32(x)
    return w.bytes()


class Rd:
    def __init__(self, d): self.d, self.p = d, 0
    def u8(self):
        v = self.d[self.p]; self.p += 1; return v
    def u16(self):
        v = int.from_bytes(self.d[self.p:self.p + 2], 'big'); self.p += 2; return v
    def u32(self):
        v = int.from_bytes(self.d[self.p:self.p + 4], 'big'); self.p += 4; return v
    def take(self, n):
        v = self.d[self.p:self.p + n]; assert len(v) == n, 'truncated'; self.p += n; return v
    def bigsmart(self):
        if self.d[self.p] & 0x80: return self.u32() & 0x7fffffff
        return self.u16()
    def ssmart(self):
        if self.d[self.p] < 128: return self.u8() - 64
        return self.u16() - 49152
    def left(self): return len(self.d) - self.p


class Wr:
    def __init__(self): self.b = bytearray()
    def u8(self, v): self.b.append(v & 255)
    def u16(self, v): self.b += (v & 0xffff).to_bytes(2, 'big')
    def u32(self, v): self.b += (v & 0xffffffff).to_bytes(4, 'big')
    def raw(self, v): self.b += v
    def bigsmart(self, v):
        if v >= 32768: self.u32(v | 0x80000000)
        else: self.u16(v)
    def ssmart(self, v):
        if -64 <= v < 64: self.u8(v + 64)
        elif -16384 <= v < 16384: self.u16(v + 49152)
        else: raise ValueError(f'value {v} outside the signed-smart range -16384..16383')
    def bytes(self): return bytes(self.b)


# ------------------------------------------------------------------------------------------ sequence definition
SEQ_DEFAULTS = dict(loop_frames=-1, interleave=None, stretch=False, priority=5, shield=-1, weapon=-1, max_loops=99,
                    precedence=None, priority_moving=None, restart=2, chat_frames=None, cached_animation=None,
                    cached_range=None, extra=None, bone_mask=None, flag19=False, sounds=None)


def decode_sequence(data):
    """Returns a dict. 'ops' keeps the opcodes in stored order so that encode_sequence is byte-exact."""
    r = Rd(data)
    s = dict(SEQ_DEFAULTS, frames=[], lengths=[], ops=[])
    while True:
        op = r.u8()
        if op == 0:
            break
        s['ops'].append(op)
        if op == 1:
            n = r.u16()
            s['lengths'] = [r.u16() for _ in range(n)]
            lo = [r.u16() for _ in range(n)]
            hi = [r.u16() for _ in range(n)]
            s['frames'] = [(h << 16) | l for h, l in zip(hi, lo)]
        elif op == 2: s['loop_frames'] = r.u16()
        elif op == 3: s['interleave'] = [r.u8() for _ in range(r.u8())]
        elif op == 4: s['stretch'] = True
        elif op == 5: s['priority'] = r.u8()
        elif op == 6: s['shield'] = r.u16()
        elif op == 7: s['weapon'] = r.u16()
        elif op == 8: s['max_loops'] = r.u8()
        elif op == 9: s['precedence'] = r.u8()
        elif op == 10: s['priority_moving'] = r.u8()
        elif op == 11: s['restart'] = r.u8()
        elif op == 12:
            n = r.u8()
            lo = [r.u16() for _ in range(n)]
            hi = [r.u16() for _ in range(n)]
            s['chat_frames'] = [(h << 16) | l for h, l in zip(hi, lo)]
        elif op == 13: s['cached_animation'] = r.u32()
        elif op == 14:
            s['sounds'] = [(r.u16(), r.u16(), r.u8(), r.u8(), r.u8(), r.u8()) for _ in range(r.u16())]
        elif op == 15: s['cached_range'] = (r.u16(), r.u16())
        elif op == 16: s['extra'] = r.u8()
        elif op == 17: s['bone_mask'] = [r.u8() for _ in range(r.u8())]
        elif op == 19: s['flag19'] = True
        else:
            raise ValueError(f'sequence opcode {op}')
    assert r.left() == 0, 'sequence trailing bytes'
    return s


def encode_sequence(s):
    """Inverse of decode_sequence. With no 'ops' key the opcodes are written in ascending order for every field
    that differs from the client's default."""
    ops = s.get('ops')
    if ops is None:
        ops = []
        if s.get('frames'): ops.append(1)
        if s.get('loop_frames', -1) != -1: ops.append(2)
        if s.get('interleave') is not None: ops.append(3)
        if s.get('stretch'): ops.append(4)
        if s.get('priority', 5) != 5: ops.append(5)
        if s.get('shield', -1) != -1: ops.append(6)
        if s.get('weapon', -1) != -1: ops.append(7)
        if s.get('max_loops', 99) != 99: ops.append(8)
        if s.get('precedence') is not None: ops.append(9)
        if s.get('priority_moving') is not None: ops.append(10)
        if s.get('restart', 2) != 2: ops.append(11)
        if s.get('chat_frames'): ops.append(12)
        if s.get('cached_animation') is not None: ops.append(13)
        if s.get('sounds'): ops.append(14)
        if s.get('cached_range') is not None: ops.append(15)
        if s.get('extra') is not None: ops.append(16)
        if s.get('bone_mask') is not None: ops.append(17)
        if s.get('flag19'): ops.append(19)
    w = Wr()
    for op in ops:
        w.u8(op)
        if op == 1:
            assert len(s['frames']) == len(s['lengths'])
            w.u16(len(s['frames']))
            for v in s['lengths']: w.u16(v)
            for v in s['frames']: w.u16(v & 0xffff)
            for v in s['frames']: w.u16(v >> 16)
        elif op == 2: w.u16(s['loop_frames'])
        elif op == 3:
            w.u8(len(s['interleave']))
            for v in s['interleave']: w.u8(v)
        elif op == 5: w.u8(s['priority'])
        elif op == 6: w.u16(s['shield'])
        elif op == 7: w.u16(s['weapon'])
        elif op == 8: w.u8(s['max_loops'])
        elif op == 9: w.u8(s['precedence'])
        elif op == 10: w.u8(s['priority_moving'])
        elif op == 11: w.u8(s['restart'])
        elif op == 12:
            w.u8(len(s['chat_frames']))
            for v in s['chat_frames']: w.u16(v & 0xffff)
            for v in s['chat_frames']: w.u16(v >> 16)
        elif op == 13: w.u32(s['cached_animation'])
        elif op == 14:
            w.u16(len(s['sounds']))
            for a, b, c, d, e, f in s['sounds']:
                w.u16(a); w.u16(b); w.u8(c); w.u8(d); w.u8(e); w.u8(f)
        elif op == 15:
            w.u16(s['cached_range'][0]); w.u16(s['cached_range'][1])
        elif op == 16: w.u8(s['extra'])
        elif op == 17:
            w.u8(len(s['bone_mask']))
            for v in s['bone_mask']: w.u8(v)
    w.u8(0)
    return w.bytes()


# --------------------------------------------------------------------------------------------------- frame base
def decode_base(data):
    """Frame base: {'types': [...], 'labels': [[...], ...], 'tail': bytes}. The tail (if any) is the bone section
    the keyframe animations use; it is carried, not interpreted."""
    r = Rd(data)
    n = r.u8()
    types = list(r.take(n))
    sizes = list(r.take(n))
    labels = [list(r.take(k)) for k in sizes]
    return dict(types=types, labels=labels, tail=data[r.p:])


def encode_base(b):
    w = Wr()
    w.u8(len(b['types']))
    w.raw(bytes(b['types']))
    w.raw(bytes(len(l) for l in b['labels']))
    for l in b['labels']: w.raw(bytes(l))
    w.raw(b.get('tail', b''))
    return w.bytes()


# -------------------------------------------------------------------------------------------------------- frame
def decode_frame(data, base):
    """Frame file -> {'base': id, 'count': stored group count, 'groups': {group index: [x, y, z]}, 'masks': {...}}.
    Only groups with a non-zero mask are listed; an axis absent from the mask holds the default (0, or 128 for
    a scale group)."""
    r = Rd(data)
    bid = r.u16()
    count = r.u8()
    masks = list(r.take(count))
    groups, mk = {}, {}
    for i, m in enumerate(masks):
        if m == 0:
            continue
        kind = base['types'][i]
        v = [128 if kind == 3 else 0] * 3
        for axis in range(3):
            if m & (1 << axis):
                v[axis] = r.ssmart()
        groups[i] = v
        mk[i] = m
    assert r.left() == 0, 'frame trailing bytes'
    return dict(base=bid, count=count, groups=groups, masks=mk)


def encode_frame(frame, base):
    """Inverse of decode_frame. Masks are taken from frame['masks'] when present (exact re-encoding of a real
    frame), otherwise an axis is written when its value differs from the default. A group whose three values
    are all default is written with mask 0, which the client skips (it is the identity)."""
    groups = frame['groups']
    masks = frame.get('masks') or {}
    count = frame.get('count')
    need = max(groups) + 1 if groups else 0
    if count is None or count < need:
        count = need
    assert count <= 255 and count <= len(base['types']), 'more groups than the frame base has'
    w = Wr()
    w.u16(frame['base'])
    w.u8(count)
    vals = Wr()
    for i in range(count):
        if i not in groups:
            w.u8(0); continue
        kind = base['types'][i]
        default = 128 if kind == 3 else 0
        v = groups[i]
        m = masks.get(i)
        if m is None:
            m = sum(1 << a for a in range(3) if v[a] != default)
        else:
            assert all(m & (1 << a) or v[a] == default for a in range(3)), 'mask drops a non-default value'
        w.u8(m)
        for a in range(3):
            if m & (1 << a):
                vals.ssmart(v[a])
    w.raw(vals.bytes())
    return w.bytes()


def transforms(frame, base):
    """The transform list the client builds from a frame (animation.rs Frame::decode): (group, [x, y, z]) in
    order, with the implied origin inserted before a non-origin group."""
    out = []
    last = -1
    for i in sorted(frame['groups']):
        kind = base['types'][i]
        if kind != 0:
            for j in range(i - 1, last, -1):
                if base['types'][j] == 0:
                    out.append((j, [0, 0, 0]))
                    break
        out.append((i, list(frame['groups'][i])))
        last = i
    return out


# ----------------------------------------------------------------------------- deformation (port of deform.rs)
import math


def _trunc_div(a, b):
    q = abs(a) // abs(b)
    return q if (a >= 0) == (b >= 0) else -q


def _i32(v):
    v &= 0xffffffff
    return v - (1 << 32) if v & 0x80000000 else v


def apply_frame(verts, labels, frame, base, mask=None, selected=False):
    """Integer port of Model::animate_classic_mask. verts: list of [x, y, z] ints (cache space, y down), modified
    in place. labels: one vertex label per vertex. mask: a sequence's interleave list (client appends 9999999)."""
    by = {}
    for i, l in enumerate(labels):
        by.setdefault(l, []).append(i)
    pivot = [0, 0, 0]
    mlist = (list(mask) + [9999999]) if mask is not None else None
    mi = 0
    for g, vals in transforms(frame, base):
        kind = base['types'][g]
        if mlist is not None:
            while g > mlist[mi]: mi += 1
            if (g == mlist[mi]) != selected and kind != 0:
                continue
        idx = [i for l in base['labels'][g] for i in by.get(l, ())]
        if kind == 0:
            if not idx:
                pivot = list(vals)
            else:
                pivot = [vals[a] + _trunc_div(sum(verts[i][a] for i in idx), len(idx)) for a in range(3)]
        elif kind == 1:
            for i in idx:
                for a in range(3): verts[i][a] += vals[a]
        elif kind == 2:
            ang = [(v & 255) * 8 for v in vals]
            trig = [(int(math.sin(a * 0.0030679615) * 65536.0), int(math.cos(a * 0.0030679615) * 65536.0)) for a in ang]
            for i in idx:
                v = verts[i]
                for a in range(3): v[a] -= pivot[a]
                for axis in (2, 0, 1):
                    if ang[axis] == 0: continue
                    s, c = trig[axis]
                    if axis == 2:
                        x, y = v[0], v[1]
                        v[0] = _i32(s * y + c * x) >> 16; v[1] = _i32(c * y - s * x) >> 16
                    elif axis == 0:
                        x, y = v[1], v[2]
                        v[1] = _i32(c * x - s * y) >> 16; v[2] = _i32(s * x + c * y) >> 16
                    else:
                        x, y = v[0], v[2]
                        v[0] = _i32(s * y + c * x) >> 16; v[2] = _i32(c * y - s * x) >> 16
                for a in range(3): v[a] += pivot[a]
        elif kind == 3:
            for i in idx:
                for a in range(3):
                    verts[i][a] = _trunc_div(vals[a] * (verts[i][a] - pivot[a]), 128) + pivot[a]
    return verts


# ------------------------------------------------------------------------------------------- convenience loaders
_cache = None


def cache():
    global _cache
    if _cache is None: _cache = Cache()
    return _cache


_seqs = None


def sequences(c=None):
    """{id: raw definition bytes} of every sequence."""
    global _seqs
    if c is not None: return c.files(2, 12)
    if _seqs is None: _seqs = cache().files(2, 12)
    return _seqs


def sequence(sid, c=None):
    return decode_sequence(sequences(c)[sid])


_bases = {}


def frame_base(bid, c=None):
    if c is not None: return decode_base(c.file(1, bid, 0))
    if bid not in _bases: _bases[bid] = decode_base(cache().file(1, bid, 0))
    return _bases[bid]


_groups = {}


def frame_group(gid, c=None):
    if c is not None: return c.files(0, gid)
    if gid not in _groups: _groups[gid] = cache().files(0, gid)
    return _groups[gid]


def sequence_frames(sid, c=None):
    """(sequence dict, base id, base, [decoded frame, ...]) for a classic sequence."""
    s = sequence(sid, c)
    out = []
    bid = base = None
    for fid in s['frames']:
        data = frame_group(fid >> 16, c)[fid & 0xffff]
        if base is None:
            bid = int.from_bytes(data[:2], 'big'); base = frame_base(bid, c)
        out.append(decode_frame(data, base))
    return s, bid, base, out
