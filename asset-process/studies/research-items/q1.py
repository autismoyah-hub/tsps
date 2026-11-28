import sys, pathlib; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import json, collections, numpy as np, statistics as st
import itemlib as L
IT=L.items(); M=json.loads((L.HERE/'data'/'measure-items.json').read_text())
for slot in ('3','0','10','7','5'):
    rs=M['inventory_vs_worn'][slot]
    print('slot',slot)
    for kind in ('moved copy','reposed copy','different mesh'):
        rr=[r for r in rs if r['kind']==kind]
        if not rr: continue
        ids=[r['id'] for r in rr]
        thin=collections.Counter(int(np.argmin(r['inv_extent'])) for r in rr)
        print(f'  {kind}: n={len(rr)} id median {st.median(ids)} (quartiles {np.percentile(ids,25):.0f}, {np.percentile(ids,75):.0f}); thin axis {dict(thin)}; origin inside {sum(r["origin_inside"] for r in rr)}')
        if kind!='different mesh':
            sh=collections.Counter(tuple(round(x) for x in r['shift']) for r in rr).most_common(5)
            print('     shifts',sh)
            print('     examples',[(r['id'],r['name'],r['angles'],r['zoom'],[round(x) for x in r['inv_extent']],[round(x) for x in r['inv_centre']]) for r in rr[-6:]])
    # by era
    for lo,hi in ((0,5000),(5000,12000),(12000,20000),(20000,26000),(26000,40000)):
        rr=[r for r in rs if lo<=r['id']<hi]
        if rr:
            c=collections.Counter(r['kind'] for r in rr)
            print(f'  ids {lo}-{hi}: n={len(rr)} '+', '.join(f'{k} {v*100//len(rr)}%' for k,v in c.most_common()))
# second head models: are they kit head models?
kit_heads=collections.Counter()
for d in IT.values():
    if L.plain(d) and d['head_models'][0][1]!=-1: kit_heads[d['head_models'][0][1]]+=1
print('second male head models most common:',kit_heads.most_common(8))
c=collections.Counter()
for d in IT.values():
    if L.plain(d) and d['wear_positions'][0]==0 and d['head_models'][0][0]!=-1:
        for g in (0,1):
            for i in d['head_models'][g]:
                if i in (63,120,81): c[(g,i)]+=1
print('helms naming a kit head model directly:',c)
# which opcodes do records use
ops=collections.Counter()
import itemcodec
