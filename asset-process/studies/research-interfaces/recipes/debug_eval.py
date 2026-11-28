"""debug_eval.py FAMILY: seed recipe of a family and its evaluation on the first instances."""
import sys, collections
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
import common as K
import census_frames as F

name = sys.argv[1]
fam = F.FAMILIES[name]
L = []
for g, rows, by in K.all_states():
    for inst in F.find_instances(rows, by, fam):
        inst['group'] = g
        inst['parent_clip'] = by[inst['parent']]['clip'] if inst['parent'] in by else [0, 0, 765, 503]
        L.append(inst)
pg = collections.Counter(i['group'] for i in L)
ksig = lambda i: tuple(sorted({(r['pk'],) + F.key_of(r) for r in i['members'] if r['sprites'][0] not in fam['optional']}))
kc = collections.Counter()
for i in L:
    kc[ksig(i)] += 1.0 / pg[i['group']]
for k, v in kc.most_common(3):
    print(round(v, 2), k)
top = kc.most_common(1)[0][0]
seed = [i for i in L if ksig(i) == top]
rec = F.infer(seed)
canon = F.canon_order(rec, seed)
print('canon', canon)
for s in canon:
    print(s, F.fmt(rec[s]['rect']), rec[s]['tiled'])
for i, ok, nd, d in F.evaluate(fam, rec, canon, seed)[:4]:
    print(i['group'], i['box'], ok, nd, d)
