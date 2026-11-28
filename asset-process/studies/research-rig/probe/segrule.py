import sys, json, pathlib, collections
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent.parent))
import numpy as np
import riglib as R, wlib
B, RM = R.B, R.RM
work = pathlib.Path(sys.argv[1]).resolve(); man = json.loads((work / 'manifest.json').read_text()); src = work / man['src']
sk = RM.Skeleton(B.decode_base((src / man['base_file']).read_bytes())); m = B.decode_model((src / man['models'][0]).read_bytes())
V = np.array(m['vertices'], float); W = m['bone_weights']; dom = np.array([max(w, key=lambda bw: bw[1])[0] for w in W]); H = sk.head()
mk = set(wlib.markers(sk)); segs = [(p, c) for c in range(sk.n) for p in [sk.parent[c]] if p >= 0 and c not in mk and np.linalg.norm(H[c] - H[p]) > 1e-6]
D = np.stack([wlib.point_segment(V, H[p], H[c])[0] for p, c in segs], 1); T = np.stack([wlib.point_segment(V, H[p], H[c])[1] for p, c in segs], 1)
k = D.argmin(1); res = collections.Counter(); tpar = []; tch = []
for i in range(len(V)):
    p, c = segs[k[i]]; t = T[i, k[i]]
    if dom[i] == p: res['parent end bone'] += 1; tpar.append(t)
    elif dom[i] == c: res['child end bone'] += 1; tch.append(t)
    elif sk.parent[dom[i]] == c: res['a child of the child'] += 1
    elif sk.parent[dom[i]] == p: res['a sibling (other child of the parent)'] += 1
    elif dom[i] == sk.parent[p]: res['grandparent'] += 1
    else: res['other'] += 1
print(man['name'], dict(res), 'of', len(V))
print('position along the nearest segment (0 = parent joint, 1 = child joint): when the PARENT bone owns the vertex, median', np.median(tpar).round(2), '; when the CHILD owns it, median', np.median(tch).round(2))
h = lambda x: np.histogram(x, bins=[0, .001, .25, .5, .75, .999, 1.0001])[0].tolist()
print(' t bins [0], (0,.25), (.25,.5), (.5,.75), (.75,1), [1]: parent-owned', h(tpar), 'child-owned', h(tch))
