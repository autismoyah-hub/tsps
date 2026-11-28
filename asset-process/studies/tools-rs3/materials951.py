"""Recover material texture references and retain every uninterpreted field."""
import sys,pathlib,collections,json
sys.path.insert(0,str(pathlib.Path(__file__).resolve().parent))
from rs3cache import *
def inspect(data):
    r=Cursor(data)
    v=r.u8()
    prefix=r.take(2).hex() if v==0 else ''
    flags=r.num(4)
    bits={0:(1,flags&10,16),1:(32,64,128)}.get(v,())
    refs=[]
    for bit in bits:
        if flags&bit:
            tag=r.u8() if v==1 else None
            refs.append([bit,tag,r.num(4)])
    if v not in (0,1):return dict(layout=v,flags=flags,references=[],raw_hex=data.hex())
    return dict(layout=v,prefix=prefix,flags=flags,references=refs,remaining_hex=data[r.pos:].hex())

if __name__=='__main__':
    c=Cache()
    tex=c.index(52)
    rows={}
    counts=collections.Counter()
    for i,d in c.files(26,0).items():
        try: row=inspect(d)
        except ValueError as e:row=dict(error=str(e),raw=d.hex())
        rows[i]=row
        for bit,tag,tid in row.get('references',[]):
            counts[row['layout'],bit,tid in tex]+=1
    (ROOT/'materials.json').write_text(json.dumps(rows,separators=(',',':')))
    print('layout',collections.Counter(d[0] for d in c.files(26,0).values()))
    print('flags',collections.Counter((v['layout'],v['flags']) for v in rows.values() if 'flags' in v).most_common(24))
    print('joins',sorted(counts.items()),'errors',sum('error' in v for v in rows.values()))

