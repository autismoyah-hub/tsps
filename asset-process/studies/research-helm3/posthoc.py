"""posthoc.py: two POST-HOC looks suggested by the renders (not pre-registered; cannot count). -> data/posthoc.json
  ph_flat_share     share of shell area drawn flat-shaded (render type 1) rather than smooth (Gouraud)
  ph_smooth_crease  share of shell edge length folded over 30 degrees where both faces are smooth-shaded and the
                    two faces share the vertex (the crease is blurred into a gradient)
"""
import sys, pathlib, json
sys.dont_write_bytecode = True
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
import h3
from h3 import h2, fitlib

out = {}
for k in h3.ALL + h3.ACCP:
    mh = h2.mesh(k); V = mh['V']
    keep = [i for i, f in enumerate(mh['F']) if len(set(f)) == 3 and not fitlib.wearer_face(mh['col'][i], mh['raw'][i])]
    T = np.array([[V[a] for a in mh['F'][i]] for i in keep]); n = np.cross(T[:, 1] - T[:, 0], T[:, 2] - T[:, 0])
    area = np.linalg.norm(n, axis=1) / 2; n = n / np.maximum(np.linalg.norm(n, axis=1)[:, None], 1e-12)
    flat = np.array([mh['flat'][i] for i in keep])
    ed = {}
    for j, i in enumerate(keep):
        f = mh['F'][i]
        for a, b in ((0, 1), (1, 2), (2, 0)):
            ed.setdefault(tuple(sorted((f[a], f[b]))), []).append(j)       # unwelded: shared vertex indices = shared normal
    tot = sc = 0.0
    for e, fs in ed.items():
        if len(fs) != 2: continue
        i, j = fs; L_ = float(np.linalg.norm(V[e[0]] - V[e[1]])); tot += L_
        ang = np.degrees(np.arccos(np.clip(n[i] @ n[j], -1, 1)))
        if ang > 30 and not flat[i] and not flat[j]: sc += L_
    out[k] = dict(ph_flat_share=round(float(area[flat == 1].sum() / area.sum()), 3), ph_smooth_crease=round(sc / max(tot, 1e-9), 3))
    print(f'{k:12} {h3.h2.H[k]["group"] if k in h3.h2.H else ""}', out[k])
json.dump(out, open(h3.DATA / 'posthoc.json', 'w'), indent=1)


# POST-HOC 3 (from the board's mesh row): crowded folds = share of hard-fold (>45) length running within 1.5 units of
# another roughly parallel hard fold that shares no vertex with it (doubled seams / thin strips)
from scipy.spatial import cKDTree
for k in h3.ALL + h3.ACCP:
    P, F = h3.shell(h2.mesh(k)); T, n, area, ed, fold, ln = h3.topo(P, F)
    hard = [e for e, a in fold.items() if a > 45]
    mid = np.array([(P[a] + P[b]) / 2 for a, b in hard]); dv = np.array([P[b] - P[a] for a, b in hard])
    dv = dv / np.linalg.norm(dv, axis=1)[:, None]; tr = cKDTree(mid); good = 0.0; tot = 0.0
    for i, e in enumerate(hard):
        tot += ln[e]
        for j in tr.query_ball_point(mid[i], 1.5):
            if j != i and not (set(hard[j]) & set(e)) and abs(dv[i] @ dv[j]) > 0.9:
                good += ln[e]; break
    out[k]['ph_crowded_folds'] = round(good / tot, 3)
    print(f'{k:12}', 'crowded', out[k]['ph_crowded_folds'])
json.dump(out, open(h3.DATA / 'posthoc.json', 'w'), indent=1)
