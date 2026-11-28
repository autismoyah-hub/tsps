"""Build 951 read-only reader for the OpenRS2 flat-file export (one folder per archive, N.dat per group).

Each N.dat is a JS5 container followed by a two-byte version trailer. Reference tables are 255/A.dat.
Nothing from the cache is executed; every length is bounded before decompression.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, lzma, zlib, bz2, itertools, struct
from pathlib import Path
from functools import lru_cache
ROOT = Path(__file__).resolve().parent
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT.parent/'tools-hd-era'))
from hdcache import Cursor
FLAT = Path(_AP.fix('<RS3_FLAT>'))

def unpack(raw, limit=128 << 20):
    """Return (payload, container_bytes_without_trailer, trailer)."""
    r = Cursor(raw)
    k, n = r.u8(), r.num(4)
    head = 5 if k == 0 else 9
    if k == 0:
        data = r.take(n)
    else:
        length = r.num(4)
        if length > limit: raise ValueError('expansion limit')
        src = r.take(n)
        if k == 1: data = bz2.BZ2Decompressor().decompress(b'BZh1'+src, length+1)
        elif k == 2: data = zlib.decompressobj(31).decompress(src, length+1)
        elif k == 3:
            if int.from_bytes(src[1:5], 'little') > 64 << 20: raise ValueError('dictionary size')
            hdr = src[:5]+length.to_bytes(8, 'little')
            data = lzma.LZMADecompressor(lzma.FORMAT_ALONE, memlimit=256 << 20).decompress(hdr+src[5:], length+1)
        else: raise ValueError(('codec', k))
        if len(data) != length: raise ValueError('length')
    end = head+n
    return data, raw[:end], raw[end:]

class Cache:
    rev = 951
    def __init__(self, path=FLAT):
        self.path = Path(path)
    def raw(self, a, g):
        p = self.path/str(a)/f'{g}.dat'
        if not p.is_file(): raise KeyError((a, g))
        return p.read_bytes()
    def group(self, a, g):
        return unpack(self.raw(a, g))[0]
    def present(self, a):
        return sorted(int(p.stem) for p in (self.path/str(a)).glob('*.dat') if p.stem.isdecimal())
    @lru_cache(maxsize=None)
    def references(self, a):
        """Per group: crc (container), digest, (packed, expanded) sizes, uncompressed crc, version, file ids."""
        r = Cursor(self.group(255, a))
        protocol = r.u8()
        if protocol not in (5, 6, 7): raise ValueError(protocol)
        if protocol >= 6: r.num(4)
        flags = r.u8()
        def big():
            return r.num(4) & 0x7fffffff if protocol == 7 and r.data[r.pos] >= 128 else r.u16()
        count = big()
        ids = list(itertools.accumulate(big() for _ in range(count)))
        if flags & 1: r.take(count*4)
        crc = [r.num(4) for _ in ids]
        plain = [r.num(4) for _ in ids] if flags & 8 else [None]*count
        dig = [r.take(64) for _ in ids] if flags & 2 else [None]*count
        sizes = [(r.num(4), r.num(4)) for _ in ids] if flags & 4 else [(None, None)]*count
        ver = [r.num(4) for _ in ids]
        nfiles = [big() for _ in ids]
        files = [list(itertools.accumulate(big() for _ in range(n))) for n in nfiles]
        if flags & 1: r.take(sum(nfiles)*4)
        if r.pos != len(r.data): raise ValueError(f'reference tail {a}')
        return {g: dict(crc=c, digest=d, sizes=s, plain=p, version=v, files=f)
                for g, c, d, s, p, v, f in zip(ids, crc, dig, sizes, plain, ver, files)}
    def index(self, a):
        return {g: row['files'] for g, row in self.references(a).items()}
    def verify(self, a, g):
        """Check the stored container against archive 255. Returns a dict of booleans."""
        raw = self.raw(a, g)
        data, container, trailer = unpack(raw)
        ref = self.references(a)[g]
        out = dict(container_crc=zlib.crc32(container) == ref['crc'])
        if ref['sizes'][0] is not None:
            out['packed'] = len(container) == ref['sizes'][0]
            out['expanded'] = len(data) == ref['sizes'][1]
        if ref['plain'] is not None:
            out['plain_crc'] = zlib.crc32(data) == ref['plain']
        if len(trailer) == 2:
            out['version_trailer'] = int.from_bytes(trailer, 'big') == ref['version'] & 0xffff
        return out
    @lru_cache(maxsize=16)
    def files(self, a, g):
        ids = self.index(a)[g]
        data = self.group(a, g)
        if len(ids) == 1: return {ids[0]: data}
        chunks = data[-1]
        end = len(data)-1-4*chunks*len(ids)
        if end < 0: raise ValueError('chunk table')
        sizes = struct.iter_unpack('>i', data[end:-1])
        p = 0
        out = {i: bytearray() for i in ids}
        for _ in range(chunks):
            size = 0
            for i in ids:
                size += next(sizes)[0]
                if size < 0 or p+size > end: raise ValueError('chunk bounds')
                out[i].extend(data[p:p+size]); p += size
        if p != end: raise ValueError('chunks incomplete')
        return {i: bytes(d) for i, d in out.items()}
