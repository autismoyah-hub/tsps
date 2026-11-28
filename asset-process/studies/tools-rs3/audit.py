"""Validate indexed chains and search for sectors unaccounted for by any live entry."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys,pathlib,json
sys.path.insert(0,str(pathlib.Path(__file__).resolve().parent))
from rs3cache import *
import numpy as np
c=Cache()
catalog=json.loads(Path(_AP.fix('<RS3_DIR>/caches.json')).read_bytes())
print(next(r for r in catalog if r['id']==2773))
data=np.frombuffer(c.disk,np.uint8)
sectors=len(data)//520
# The final sector may be partial. Pad that sector for header checks only.
def col(offs):
    return data[offs::520]
seen=np.zeros((len(data)+519)//520,dtype=bool)
seen[0]=True
report={}
for a,idx in sorted(c.idx.items()):
    table=np.frombuffer(idx,np.uint8).reshape(-1,6).astype(np.uint32)
    sizes=(table[:,0]*256+table[:,1])*256+table[:,2]
    starts=(table[:,3]*256+table[:,4])*256+table[:,5]
    ids=np.where(sizes>0)[0]
    refs=c.index(a) if a!=255 else {}
    report[a]=dict(idx_bytes=len(idx),advertised=len(refs),files=sum(map(len,refs.values())),stored=len(ids),missing=len(refs.keys()-set(ids)))
    for wide in (False,True):
        gs=ids[(ids>65535)==wide]
        width=2+2*wide
        room=514-width
        remaining=sizes[gs].astype(np.int64)
        pos=starts[gs]
        seq=0
        while len(gs):
            valid=(pos>0)&(pos<len(seen))
            if not np.all(valid):
                report[a].setdefault('broken_chains',[]).extend([int(g),int(s),seq] for g,s in zip(gs[~valid],pos[~valid]))
                gs,remaining,pos=gs[valid],remaining[valid],pos[valid]
                if not len(gs): break
            if np.any(seen[pos]): raise ValueError(('reused',a))
            seen[pos]=True
            def num(i,n):
                v=np.zeros(len(gs),np.uint32)
                for k in range(n): v=v*256+col(i+k)[pos]
                return v
            if np.any(num(0,width)!=gs) or np.any(num(width,2)!=seq) or np.any(num(width+5,1)!=a): raise ValueError(('header',a))
            nxt=num(width+2,3)
            remaining-=room
            live=remaining>0
            if np.any(nxt[~live]): raise ValueError(('tail',a))
            gs,remaining,pos=gs[live],remaining[live],nxt[live]
            seq+=1
print('unaccounted',np.count_nonzero(~seen),'of',len(seen))
(ROOT/'disk-audit.json').write_text(json.dumps(dict(bytes=len(data),sector_slots=len(seen),unaccounted=int(np.count_nonzero(~seen)),archives=report),indent=2))

