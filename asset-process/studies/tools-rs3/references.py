"""Identify duplicate compressed containers using reference digests without reading missing groups."""
import sys,pathlib,json,collections,hashlib,zlib
sys.path.insert(0,str(pathlib.Path(__file__).resolve().parent))
from rs3cache import *
def headers(c,a):
    r=Cursor(c.group(255,a))
    protocol=r.u8()
    if protocol>=6:r.num(4)
    flags=r.u8()
    def big():return r.num(4)&0x7fffffff if protocol==7 and r.data[r.pos]>=128 else r.u16()
    count=big()
    import itertools
    ids=list(itertools.accumulate(big() for _ in range(count)))
    if flags&1:r.take(count*4)
    crc=[r.num(4) for _ in ids]
    plain=[r.num(4) for _ in ids] if flags&8 else [None]*count
    dig=[r.take(64) for _ in ids] if flags&2 else [None]*count
    sizes=[(r.num(4),r.num(4)) for _ in ids] if flags&4 else [(None,None)]*count
    return flags,dict(zip(ids,zip(crc,dig,sizes,plain)))
if __name__=='__main__':
    c=Cache()
    present={}
    plain_present={}
    report={}
    for a in sorted(c.idx.keys()-{255}):
        f,rows=headers(c,a)
        ix=c.idx.get(a,b'')
        for g,k in rows.items():
            if any(ix[6*g:6*g+3]):
                present[k]=(a,g)
                plain_present[(k[2][1],k[3])]=(a,g)
        report[a]=dict(flags=f,groups=len(rows),files=sum(map(len,c.index(a).values())))
    duplicates={}
    plain_duplicates={}
    for a in [47,52,53,54,55]:
        flags,rows=headers(c,a)
        duplicates[a]=[(g,present[k]) for g,k in rows.items() if k in present]
        plain_duplicates[a]=[(g,plain_present[(k[2][1],k[3])]) for g,k in rows.items() if (k[2][1],k[3]) in plain_present]
    print({a:len(d) for a,d in duplicates.items()})
    print('uncompressed candidates',{a:len(d) for a,d in plain_duplicates.items()})
    (ROOT/'references.json').write_text(json.dumps(dict(indexes=report,duplicate_candidates=duplicates,plain_candidates=plain_duplicates),indent=1))

