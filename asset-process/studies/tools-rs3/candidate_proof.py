"""Find exact raw-byte candidates by uncompressed size and CRC across local comparison disks."""
import sys,pathlib,collections,json,zlib
sys.path.insert(0,str(pathlib.Path(__file__).resolve().parent))
from references import *
c=Cache()
_,rows=headers(c,47)
keys=collections.defaultdict(list)
for g,(_,_,sizes,plain) in rows.items():keys[sizes[1],plain].append(g)
lengths={k[0] for k in keys}
out={}
for rev in (239,561,601,614):
    h=HDCache(rev)
    candidates=[]
    for g in h.index(7):
        data=h.group(7,g)
        if len(data) in lengths:
            for other in keys.get((len(data),zlib.crc32(data)),[]):candidates.append([g,other])
    out[rev]=dict(checked=len(h.index(7)),candidates=candidates)
    print(rev,len(candidates),flush=True)
    (ROOT/'candidate-proof.json').write_text(json.dumps(out,indent=1))

