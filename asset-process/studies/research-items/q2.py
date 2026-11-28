import sys, pathlib; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import json, collections, numpy as np, statistics as st
import itemlib as L
import measure_fit as F
IT=L.items(); M=json.loads((L.HERE/'data'/'measure-items.json').read_text())
for slot in ('3','0','4','5','7','10'):
    rs=[r for r in M['inventory_vs_worn'][slot] if r['kind'] in('reposed copy',) or (r['kind']=='moved copy' and not 32400<r['id']<32800)]
    print('slot',L.SLOT_NAMES[int(slot)],len(rs))
    c=collections.Counter()
    ex={}
    for r in rs:
        d=IT[r['id']]; inv=L.model(d['model']); worn=L.concat(L.worn_ids(d,0))
        rms,mx,R,t,s=F.rigid_fit(worn['V'],inv['V'])
        key=tuple(np.round(R,1).flatten().tolist())
        c[key]+=1; ex.setdefault(key,[]).append((r['id'],r['name'],d['zoom'],d['angles'],d['offset'],[round(x) for x in r['inv_centre']],[round(x) for x in r['inv_extent']], inv['V'][:,1].max()))
    for key,n in c.most_common(4):
        print('  R rows (inv = worn @ R + t):',np.array(key).reshape(3,3).tolist(),'x',n)
        for e in ex[key][-4:]: print('      ',e)
