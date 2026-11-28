import sys, pathlib; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import json, collections, numpy as np
import itemlib as L
IT=L.items(); M=json.loads((L.HERE/'data'/'measure-items.json').read_text())
for slot in ('0','4','7','9','10','1'):
    rs=M['female_vs_male'][slot]
    dm=[r for r in rs if r['kind']=='different mesh']
    eqv=0; eqf=0
    for r in dm:
        a=L.concat(r['male']); b=L.concat(r['female'])
        eqv+=len(a['V'])==len(b['V']); eqf+=len(a['F'])==len(b['F'])
    print(L.SLOT_NAMES[int(slot)],'different mesh',len(dm),'equal vertex count',eqv,'equal face count',eqf)
    # frequent coordinates per gender (distinct meshes only)
    for g,key in ((0,'male'),(1,'female')):
        seen=set(); cnt=collections.Counter(); lab=collections.defaultdict(collections.Counter); n=0
        for r in rs:
            ids=tuple(r[key])
            if not ids or ids in seen: continue
            seen.add(ids); m=L.concat(list(ids))
            if m is None: continue
            n+=1
            off=r['off'][g]
            pts={(int(v[0]),int(-(v[1]+off)),int(v[2])):l for v,l in zip(m['V'],m['vertex_skins'])}
            for p,l in pts.items(): cnt[p]+=1; lab[p][l]+=1
        top=[(p,round(c*100/n),lab[p].most_common(2)) for p,c in cnt.most_common(14)]
        print(f'  {key}: {n} distinct meshes; most shared coordinates (x,height,z), % of meshes, labels:')
        for t in top: print('     ',t)
L.save_models()
