import sys,pathlib
sys.path.insert(0,str(pathlib.Path(__file__).parent))
from audit import *
o=Cache(239);lookup=collections.defaultdict(list)
for i in o.index(7):
 try:lookup[signature(o.group(7,i))].append(i)
 except (IndexError,struct.error):continue
h=Cache(561);out=[]
for i in h.index(7):
 if i<40000:continue
 raw=h.group(7,i)
 for j in lookup.get(signature(raw),[]):
    m=decode(raw)
    if len(m['faces'])<20 or any(t>=0 for t in m['face_textures']):break
    n=osrs.decode_model(o.group(7,j))
    if all(m[k]==n[k] for k in ('vertices','faces','colors','alpha','priorities','render_types','depth_bias','face_textures')):
        out.append((i,j,len(m['vertices']),len(m['faces'])))
        break
(ROOT/'later-matches.json').write_text(json.dumps(out))
print(len(out),out[:35])

