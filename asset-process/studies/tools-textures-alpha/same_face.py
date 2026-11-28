import json,sys
from pathlib import Path
here=Path(__file__).resolve().parent
sys.path.insert(0,str(here.parents[1]/'tools'))
import osrscache as o
seen=set()
for kind,i,name,mid,t,a in json.loads((here/'candidates.json').read_bytes()):
    if not (kind in ('shape-0','shape-4','shape-10','shape-22') and t and a and mid not in seen):continue
    m=o.model(mid)
    n=sum(t>=0 and 0<a<254 for t,a in zip(m['face_textures'],m['alpha']))
    if n:print(kind,i,name,mid,n,t,a)
    seen.add(mid)
