import sys,pathlib,hashlib
sys.path.insert(0,str(pathlib.Path(__file__).parent))
from hdview import *
import numpy as np
o=Cache(239);proof=[]
for rev in (561,601,614):
 h=Cache(rev)
 pairs=[x for x in json.loads((ROOT/'matches.json').read_text()) if x[0]==rev][:5]
 if rev==561:pairs+=[[561,*row] for row in json.loads((ROOT/'later-matches.json').read_text()) if row[0] in (44486,44489,44579,44584,44595,44607,46657)]
 for r,i,j,nv,nf in pairs:
    m=decode(h.group(7,i));n=osrs.decode_model(o.group(7,j))
    assert all(m[k]==n[k] for k in ('vertices','faces','colors'))
    _,a,_=views(h,'model',i);_,b,_=views(o,'model',j)
    dif=[int(np.any(np.asarray(x)!=np.asarray(y),axis=2).sum()) for x,y in zip(a,b)]
    ink=[int(np.any(np.asarray(x)!=[38,42,48],axis=2).sum()) for x in a]
    assert sum(ink)>0 and sum(dif)==0,(rev,i,dif,ink)
    proof.append(dict(rev=rev,hd=i,osrs=j,verts=nv,faces=nf,differing=dif,foreground=ink,
                      rgb_sha256=hashlib.sha256(b''.join(x.tobytes() for x in a)).hexdigest()))
    if len(proof)==3:sheet(h,'model',i,ROOT/'proof-sheet.png')
 (ROOT/'pixel-proof.json').write_text(json.dumps(proof,indent=2))
print(len(proof),'pairs passed')

