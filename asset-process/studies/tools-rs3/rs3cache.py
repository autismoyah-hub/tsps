"""Build 951 read-only JS5 disk reader; reference-table and chunk layout from hdcache."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, mmap, lzma, zlib, bz2
from pathlib import Path
ROOT = Path(__file__).resolve().parent
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT.parent/'tools-hd-era'))
from hdcache import Cache as HDCache, Cursor

class Cache(HDCache):
    def __init__(self):
        self.rev = 951
        self.path = Path(_AP.fix('<RS3_CACHE>'))
        with (self.path/'main_file_cache.dat2').open('rb') as f:
            self.disk = mmap.mmap(f.fileno(),0,access=mmap.ACCESS_READ)
        self.idx = {int(p.suffix[4:]):p.read_bytes() for p in self.path.glob('*.idx*')}

    def group(self,a,g):
        entry=self.idx[a][6*g:6*g+6]
        size=int.from_bytes(entry[:3],'big')
        sector=int.from_bytes(entry[3:],'big')
        if not size: raise KeyError((a,g))
        out=bytearray()
        width=4 if g>65535 else 2
        for seq in range((size+513-width)//(514-width)):
            pos=sector*520
            block=self.disk[pos:pos+520]
            r=Cursor(block)
            if (r.num(width),r.u16())!=(g,seq): raise ValueError(('chain',a,g,seq))
            sector=r.num(3)
            if r.u8()!=a: raise ValueError('sector archive')
            out.extend(r.take(min(514-width,size-len(out))))
        if len(out)!=size: raise ValueError('short group')
        r=Cursor(bytes(out))
        k,n=r.u8(),r.num(4)
        if k==0: return r.take(n)
        length=r.num(4)
        if length>128<<20: raise ValueError('expansion limit')
        src=r.take(n)
        if k==1: data=bz2.BZ2Decompressor().decompress(b'BZh1'+src,length+1)
        elif k==2: data=zlib.decompressobj(31).decompress(src,length+1)
        elif k==3:
            if int.from_bytes(src[1:5],'little')>64<<20: raise ValueError('dictionary size')
            hdr=src[:5]+length.to_bytes(8,'little')
            data=lzma.LZMADecompressor(lzma.FORMAT_ALONE,memlimit=128<<20).decompress(hdr+src[5:],length+1)
        else: raise ValueError(('codec',k))
        if len(data)!=length: raise ValueError('length')
        return data

