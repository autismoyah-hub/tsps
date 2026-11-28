"""mesh_cause.py: step 2, what in the MESH produces the divisions. For each piece of each set, every interior edge
(two faces sharing an edge by coordinates) is classed:
  material  the two faces are different materials (hue or saturation differ): a trim or panel of a second material
  dark      same material, painted lightness differs by 12 or more and the darker face is painted <= 20: a painted
            dark line or recess
  tone      same material, painted lightness differs by 12 or more, both above 20: a painted tone step
  fold      same paint (within 12), the faces meet at 30 degrees or more: a change of plane, which only the light draws
  smooth    the rest
Lengths are given per square root of the piece's surface area (so pieces of different size compare).
Also: dark_area = share of surface painted lightness <= 20; L_min, L_med, L_max = painted lightness range by area;
n_mat = materials with 3 % or more of the surface.
usage: mesh_cause.py -> mesh-cause.json"""
import sys, pathlib, json, math
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
import lib as L


def same_mat(a, b):
    if a[1] == 0 and b[1] == 0: return True
    dh = abs(a[0] - b[0]); dh = min(dh, 64 - dh)
    return dh <= 2 and abs(a[1] - b[1]) <= 1


def cause(m):
    V = m['V']; edges = {}; nrm = []; area = []
    for fi, f in enumerate(m['F']):
        a, b, c = V[f[0]], V[f[1]], V[f[2]]
        n = np.cross(b - a, c - a); ln = np.linalg.norm(n); area.append(ln / 2); nrm.append(n / ln if ln else None)
        for i in range(3):
            pa, pb = tuple(V[f[i]]), tuple(V[f[(i + 1) % 3]])
            if pa != pb: edges.setdefault(tuple(sorted((pa, pb))), []).append(fi)
    area = np.array(area); tot = area.sum()
    out = dict(material=0.0, dark=0.0, tone=0.0, fold=0.0, smooth=0.0)
    for (pa, pb), fs in edges.items():
        if len(fs) != 2 or nrm[fs[0]] is None or nrm[fs[1]] is None: continue
        ln = math.dist(pa, pb); c1, c2 = m['col'][fs[0]], m['col'][fs[1]]
        ang = math.degrees(math.acos(float(np.clip(np.dot(nrm[fs[0]], nrm[fs[1]]), -1, 1))))
        if not same_mat(c1, c2): k = 'material'
        elif abs(c1[2] - c2[2]) >= 12: k = 'dark' if min(c1[2], c2[2]) <= 20 else 'tone'
        elif ang >= 30: k = 'fold'
        else: k = 'smooth'
        out[k] += ln
    r = {k: round(v / tot ** 0.5, 2) for k, v in out.items()}
    Ls = np.array([c[2] for c in m['col']]); o_ = np.argsort(Ls); cw = np.cumsum(area[o_]) / tot
    r['dark_area'] = round(float(area[Ls <= 20].sum() / tot), 3)
    r['L_p05'] = int(Ls[o_][np.searchsorted(cw, 0.05)]); r['L_med'] = int(Ls[o_][np.searchsorted(cw, 0.5)]); r['L_p95'] = int(Ls[o_][min(np.searchsorted(cw, 0.95), len(Ls) - 1)])
    mats = []
    for i, c in enumerate(m['col']):
        for mm in mats:
            if same_mat(mm[0], c): mm[1] += area[i]; break
        else: mats.append([c, area[i]])
    r['n_mat'] = sum(1 for mm in mats if mm[1] / tot >= 0.03)
    r['faces'] = len(m['F']); r['area'] = round(float(tot))
    return r


pieces = [('torva body', L.mesh_real(26384)), ('torva legs', L.mesh_real(26386)), ('torva helm', L.mesh_real(26382)),
          ('justiciar body', L.mesh_real(22327)), ('justiciar legs', L.mesh_real(22328)), ('justiciar helm', L.mesh_real(22326)),
          ('bandos body', L.mesh_real(11832)), ('bandos legs', L.mesh_real(11834)), ('serpentine helm', L.mesh_real(12931)), ('faceguard helm', L.mesh_real(24271)),
          ('oathplate body', L.mesh_real(30753)), ('oathplate legs', L.mesh_real(30756)), ('oathplate helm', L.mesh_real(30750)),
          ('earlier body', L.mesh_ours(L.CUIRASS_V2)), ('earlier legs', L.mesh_ours(f'{L.BB}/meridian-legs/meridian-legs-labelled.bin')),
          ('earlier gauntlets', L.mesh_ours(f'{L.BB}/meridian-gauntlets/meridian-gauntlets-labelled.bin')), ('earlier boots', L.mesh_ours(f'{L.BB}/meridian-boots/meridian-boots-labelled.bin')),
          ('earlier helm v2a', L.mesh_ours(L.OURS[1][1])),
          ('redesigned body', L.mesh_ours(L.CUIRASS_V3)), ('redesigned legs', L.mesh_ours(f'{L.BB}/meridian-legs-v3/meridian-legs-v3-labelled.bin')),
          ('redesigned gauntlets', L.mesh_ours(f'{L.BB}/meridian-gauntlets-v3/meridian-gauntlets-v3-labelled.bin')), ('redesigned boots', L.mesh_ours(f'{L.BB}/meridian-boots-v3/meridian-boots-v3-labelled.bin')),
          ('redesigned helm v3a', L.mesh_ours(L.OURS[4][1]))]
cols = ['faces', 'area', 'material', 'dark', 'tone', 'fold', 'smooth', 'dark_area', 'L_p05', 'L_med', 'L_p95', 'n_mat']
res = {}
print(f"{'piece':22}" + ''.join(f'{c:>10}' for c in cols) + '   painted (material+dark+tone)')
for name, m in pieces:
    r = cause(m); res[name] = r
    print(f'{name:22}' + ''.join(f'{r[c]:>10}' for c in cols) + f"   {round(r['material'] + r['dark'] + r['tone'], 2)}")
json.dump(res, open(pathlib.Path(__file__).parent / 'mesh-cause.json', 'w'), indent=1)
