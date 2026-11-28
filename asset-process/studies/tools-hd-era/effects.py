import sys,pathlib,json,collections
sys.path.insert(0,str(pathlib.Path(__file__).parent))
from definitions import *
from models import decode
c=Cache(614)
def frames(i):
 a,s=BANK['sequence'];raw=c.files(a,i>>s)[i%2**s];r=Cursor(raw)
 while op:=r.u8():
    if op==1:
        n=r.u16();r.take(2*n);low=[r.u16() for _ in range(n)];high=[r.u16() for _ in range(n)]
        return n, set(high),{int.from_bytes(c.files(0,g)[f][1:3],'big') for g,f in zip(high,low)}
    sizes={4:0,14:0,15:0,16:0,5:1,8:1,9:1,10:1,11:1,2:2,6:2,7:2}
    if op==3:r.take(r.u8())
    else:r.take(sizes[op])
 return 0,set(),set()
study=json.loads((ROOT/'study.json').read_text())
npc={}
for d in study:
 if d['kind']=='npc':
    npc[d['id']]=[frames(v[0]) for v in d['animations'].values()]
assoc=[]
for i,raw in records(c,'spot'):
 d=parse('spot',raw)
 if 'sequence' not in d or not d['models']:continue
 n,groups,bases=frames(d['sequence'])
 for nid,anim in npc.items():
    common=set.union(set(),*(g for _,g,_ in anim)) & groups
    skeleton=set.union(set(),*(b for _,_,b in anim)) & bases
    if common or skeleton:assoc.append(dict(npc=nid,spot=i,models=d['models'],sequence=d['sequence'],frames=n,groups=sorted(common),bases=sorted(skeleton)))
(ROOT/'effect-links.json').write_text(json.dumps(assoc,indent=1))
print(len(assoc),collections.Counter(d['npc'] for d in assoc),assoc[:5])

