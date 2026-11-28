import sys, pathlib; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import json, collections, numpy as np
import itemlib as L
IT=L.items(); K=json.loads((L.HERE/'data'/'kinds.json').read_text())
for kind in ('platebody','platelegs','full helm','boots','gloves/gauntlets','longsword','halberd','staff'):
    pr=collections.Counter(); dep=0; n=0; al=0
    for i in K[kind]:
        m=L.model(IT[i]['model'])
        if m is None: continue
        n+=1; pr[tuple(sorted(set(m['priorities'])))]+=1; dep+=any(m['depth_bias']); al+=any(m['alpha'])
    print(kind,n,'inventory model priorities',pr.most_common(4),'with depth offsets',dep,'alpha',al)
for i in (1127,1079,1163,4131,7462,1303,3204,1381,11802,22978, 24417, 26219, 13652):
    d=IT[i]; m=L.model(d['model']); w=L.concat(L.worn_ids(d,0))
    V=m['V']
    print(i,d['name'],'inv',d['model'],len(m['F']),'f; box',V.min(0).tolist(),V.max(0).tolist(),'zoom',d['zoom'],d['angles'],d['offset'],'amb/con',d['ambient'],d['contrast'],'| worn',len(w['F']),'f box',w['V'].min(0).tolist(),w['V'].max(0).tolist(), 'f-off',d['wear_offset'])
# weapon moved-copy shifts by kind
M=json.loads((L.HERE/'data'/'measure-items.json').read_text())
import re
for pat in ('staff','halberd|spear|glaive','sword|scimitar'):
    c=collections.Counter(); offs=collections.Counter()
    for r in M['female_vs_male']['3']:
        if re.search(pat,r['name'].lower()):
            if r['kind']=='moved copy': c[tuple(r['shift'])]+=1
            elif r['kind'].startswith('same ids'): offs[tuple(r['off'])]+=1
    print(pat,'moved copies',c.most_common(5),'| same ids offsets',offs.most_common(4))
