"""Read local JS5 disks. No game code or XTEA keys are used."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
from pathlib import Path
from functools import lru_cache
import struct, mmap, bz2, zlib, sys, itertools
ROOT = Path(__file__).resolve().parent
sys.dont_write_bytecode = True
sys.path.insert(0, str(ROOT.parents[1] / 'tools'))
import osrscache as osrs

class Cursor:
    def __init__(self, data):
        self.data, self.pos = data, 0
    def take(self, n):
        if n < 0 or self.pos+n > len(self.data):
            raise ValueError(f'truncated at {self.pos}, requested {n}')
        p=self.pos
        self.pos+=n
        return self.data[p:self.pos]
    def num(self, n):
        return int.from_bytes(self.take(n),'big')
    def u8(self): return self.num(1)
    def u16(self): return self.num(2)
    def string(self):
        p=self.data.index(0,self.pos)
        return self.take(p-self.pos+1)[:-1].decode('cp1252','replace')

class Cache:
    def __init__(self, rev):
        self.rev=rev
        self.path=(osrs.CACHE if rev==239 else Path(_AP.fix('<OSRS_CACHE_242>'))
                   if rev==242 else Path(_AP.fix(f'<HD_CACHES>/r{rev}/cache/cache')))
        with (self.path/'main_file_cache.dat2').open('rb') as f:
            self.disk=mmap.mmap(f.fileno(),0,access=mmap.ACCESS_READ)
        self.idx={int(p.suffix[4:]):p.read_bytes() for p in self.path.glob('*.idx*')}
    def group(self,a,g):
        e=self.idx[a][g*6:g*6+6]
        n,s=int.from_bytes(e[:3],'big'),int.from_bytes(e[3:],'big')
        if not 0<n<=64<<20: raise KeyError((a,g))
        out=bytearray()
        for i in itertools.count():
            h=10 if g>65535 else 8
            b=self.disk[s*520:(s+1)*520]
            if len(b)<h or (int.from_bytes(b[:h-6],'big'),int.from_bytes(b[h-6:h-4],'big'),b[h-1])!=(g,i,a):
                raise ValueError(f'bad sector {(a,g,i,s)}')
            out.extend(b[h:h+min(n-len(out),520-h)])
            s=int.from_bytes(b[h-4:h-1],'big')
            if len(out)==n: break
        c=Cursor(bytes(out))
        k,n=c.u8(),c.num(4)
        if k==0:return c.take(n)
        size=c.num(4)
        if size>64<<20: raise ValueError('expanded size')
        src=c.take(n)
        if k==1: data=bz2.BZ2Decompressor().decompress(b'BZh1'+src,max_length=size+1)
        elif k==2: data=zlib.decompressobj(31).decompress(src,size+1)
        else: raise ValueError(f'compression {k}')
        if len(data)!=size: raise ValueError('expanded length')
        return data
    @lru_cache(maxsize=200)
    def index(self,a):
        r=Cursor(self.group(255,a))
        version=r.u8()
        if version not in (5,6,7): raise ValueError(version)
        if version>=6:r.take(4)
        flags=r.u8()
        def integer():
            if version<7 or r.data[r.pos]<128:return r.u16()
            return r.num(4)&0x7fffffff
        n=integer()
        groups=list(itertools.accumulate(integer() for _ in range(n)))
        r.take(n*(4+4*bool(flags&1)+64*bool(flags&2)+8*bool(flags&4)+4*bool(flags&8)+4))
        counts=[integer() for _ in groups]
        files={g:list(itertools.accumulate(integer() for _ in range(c))) for g,c in zip(groups,counts)}
        if flags&1:r.take(sum(counts)*4)
        if r.pos!=len(r.data):raise ValueError(f'index tail {a}')
        return files
    @lru_cache(maxsize=16)
    def files(self,a,g):
        ids=self.index(a)[g]
        data=self.group(a,g)
        if len(ids)==1:return {ids[0]:data}
        chunks=data[-1]
        end=len(data)-1-4*chunks*len(ids)
        if end<0:raise ValueError('chunk table')
        sizes=struct.iter_unpack('>i',data[end:-1])
        p=0
        out={i:bytearray() for i in ids}
        for _ in range(chunks):
            size=0
            for i in ids:
                size+=next(sizes)[0]
                if size<0 or p+size>end:raise ValueError('chunk bounds')
                out[i].extend(data[p:p+size]); p+=size
        if p!=end:raise ValueError('chunks incomplete')
        return {i:bytes(d) for i,d in out.items()}

if __name__=='__main__':
    import collections,json
    stats={}
    for rev in [561,601,614,239]:
        cache=Cache(rev)
        indexes={}
        for a in sorted(cache.idx):
            if a==255:continue
            try:
                ix=cache.index(a)
                indexes[a]=(len(ix),sum(map(len,ix.values())))
            except Exception as e:indexes[a]=str(e)
        print(rev,indexes,flush=True)
        if rev==239:break
        formats=collections.Counter()
        examples={}
        for i in cache.index(7):
            d=cache.group(7,i)
            key=('extended f%d v%d'%(d[-18],d[-24] if d[-18]&8 else 12)) if d.endswith(b'\xff\xff') else 'legacy'
            formats[key]+=1
            examples.setdefault(key,i)
        stats[rev]=dict(indexes=indexes,formats=formats,examples=examples)
        print(formats,examples,flush=True)
        (ROOT/'inventory.json').write_text(json.dumps(stats,indent=2))

