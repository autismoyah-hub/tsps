"""Build 951 archive-47 model decoder (format byte 2, shared vertex buffer, indexed render groups).

Layout established from the data in this cache (see MODELS.md). Little-endian unless noted.

header  u8 format (2), u8 sub (5), u8 version, u8 group_count,
        u8 count0, u8 count39, u8 count50, u8 count18, u8 pad,
        u32 vertex_flags, u32 vertex_count
vertex streams in this order (vertex_count entries each):
        position  3 x s16; with flag 0x100, 3 x float32 big-endian
        normal    3 x s8
        tangent   4 x s8 (w is handedness, +-127)
        uv        2 x float16, big-endian
        [flag 0x08]  u16 per vertex
        [flag 0x20]  skin: u16 n, n x u16 bone, u16 n, n x u8 weight (last stored 0 = remainder of 256)
        colour    u16 HSL
        [flag 0x01]  alpha u8
        [flag 0x04]  u16 per vertex
        flag 0x40 adds no vertex stream
render groups (group_count): u8 group_flags, u8 a, u8 b, u8 c, u8 d, u16 material+1, u8 e,
        u16 index_count, index_count x u16 vertex index (triangle list)
tail: count39 x 39 bytes, count50 x 50 bytes, count18 x 18 bytes, count0 x ? (raises if nonzero)
"""
import struct
import numpy as np

KNOWN_VERTEX_FLAGS = 0x01 | 0x04 | 0x08 | 0x20 | 0x40 | 0x100
TAIL = ((5, 39), (6, 50), (7, 18))

class FormatError(ValueError):
    pass

def half_be(raw, n):
    return np.frombuffer(raw, dtype='>f2', count=2*n).astype(np.float32).reshape(n, 2)

def decode(data, keep_raw=False):
    d = memoryview(data)
    if len(d) < 17: raise FormatError('short header')
    fmt, sub, version, ngroups = d[0], d[1], d[2], d[3]
    counts = bytes(d[4:9])
    flags, nv = struct.unpack_from('<II', d, 9)
    if fmt != 2: raise FormatError(f'format byte {fmt}')
    if flags & ~KNOWN_VERTEX_FLAGS: raise FormatError(f'unknown vertex flags 0x{flags & ~KNOWN_VERTEX_FLAGS:x}')
    if counts[0] or counts[4]: raise FormatError(f'unknown header counts {counts.hex()}')
    p = 17
    def take(n):
        nonlocal p
        if n < 0 or p+n > len(d): raise FormatError(f'truncated at {p} wanting {n}')
        out = d[p:p+n]; p += n
        return out
    m = dict(version=version, sub=sub, flags=flags, header=bytes(d[:17]))
    if flags & 0x100:
        m['positions'] = np.frombuffer(take(12*nv), '>f4').reshape(nv, 3).astype(np.float64)
    else:
        m['positions'] = np.frombuffer(take(6*nv), '<i2').reshape(nv, 3).astype(np.int32)
    m['normals'] = np.frombuffer(take(3*nv), 'i1').reshape(nv, 3).astype(np.int32)
    m['tangents'] = np.frombuffer(take(4*nv), 'i1').reshape(nv, 4).astype(np.int32)
    m['uv'] = half_be(take(4*nv), nv)
    m['uv_raw'] = np.frombuffer(d[p-4*nv:p], '>u2').reshape(nv, 2)
    if flags & 0x08:
        m['stream08'] = np.frombuffer(take(2*nv), '<u2').astype(np.int32)
    if flags & 0x20:
        bones, weights = [], []
        start = p
        for _ in range(nv):
            n = struct.unpack('<H', take(2))[0]
            b = struct.unpack(f'<{n}H', take(2*n))
            k = struct.unpack('<H', take(2))[0]
            if k != n: raise FormatError(f'skin counts differ {n} {k} at {p}')
            w = list(take(k))
            if n:
                if w[-1] != 0 or sum(w[:-1]) > 256: raise FormatError(f'skin weights {w}')
                w[-1] = 256-sum(w[:-1])
            bones.append(b); weights.append(w)
        m['bones'], m['weights'] = bones, weights
        m['skin_raw'] = bytes(d[start:p])
    m['colours'] = np.frombuffer(take(2*nv), '<u2').astype(np.int32)
    if flags & 0x01:
        m['alpha'] = np.frombuffer(take(nv), 'u1').astype(np.int32)
    if flags & 0x04:
        m['stream04'] = np.frombuffer(take(2*nv), '<u2').astype(np.int32)
    # Index width: u16 little-endian, or u32 big-endian when the vertex count exceeds 65535.
    width, dtype = (4, '>u4') if nv > 65535 else (2, '<u2')
    tail_size = sum(size*counts[pos-4] for pos, size in TAIL)
    def parse_groups(q, left):
        """The 16-bit count can overflow; recover the high part by requiring an exact end."""
        if left == 0:
            return [] if q + tail_size == len(d) else None
        if q + 10 > len(d): return None
        h = bytes(d[q:q+10])
        low = int.from_bytes(h[8:10], 'little')
        for k in range(0, 256):
            count = low + 65536*k
            end = q + 10 + width*count
            if end + tail_size > len(d): break
            if count % 3: continue
            idx = np.frombuffer(d[q+10:end], dtype).astype(np.int64)
            if count and idx.max() >= nv: continue
            rest = parse_groups(end, left-1)
            if rest is not None:
                g = dict(flags=h[0], head=h[1:5], material=int.from_bytes(h[5:7], 'little')-1,
                         byte7=h[7], triangles=idx.reshape(-1, 3), count_overflow=k)
                return [g] + rest
        return None
    groups = parse_groups(p, ngroups)
    if groups is None:
        raise FormatError(f'render groups do not parse to the end from {p}')
    m['groups'] = groups
    m['index_width'] = width
    p = len(d) - tail_size
    tail = {}
    for pos, size in TAIL:
        tail[size] = [bytes(take(size)) for _ in range(counts[pos-4])]
    m['tail'] = tail
    if p != len(d): raise FormatError(f'{len(d)-p} trailing bytes at {p}')
    return m


def encode(m):
    """Re-encode a decoded model (requires the original header for pad bytes)."""
    out = bytearray(m['header'])
    nv = len(m['positions'])
    out += m['positions'].astype('>f4' if m['flags'] & 0x100 else '<i2').tobytes()
    out += m['normals'].astype('i1').tobytes()
    out += m['tangents'].astype('i1').tobytes()
    out += m['uv_raw'].astype('>u2').tobytes()
    f = m['flags']
    if f & 0x08: out += m['stream08'].astype('<u2').tobytes()
    if f & 0x20:
        for b, w in zip(m['bones'], m['weights']):
            out += struct.pack(f'<H{len(b)}HH', len(b), *b, len(w))
            if w: out += bytes(w[:-1]) + b'\0'
    out += m['colours'].astype('<u2').tobytes()
    if f & 0x01: out += m['alpha'].astype('u1').tobytes()
    if f & 0x04: out += m['stream04'].astype('<u2').tobytes()
    for g in m['groups']:
        t = g['triangles'].reshape(-1)
        out += bytes([g['flags']]) + g['head'] + (g['material']+1).to_bytes(2, 'little') + bytes([g['byte7']])
        out += (len(t) & 0xffff).to_bytes(2, 'little') + t.astype('>u4' if nv > 65535 else '<u2').tobytes()
    for _, size in TAIL:
        for rec in m['tail'][size]: out += rec
    return bytes(out)
