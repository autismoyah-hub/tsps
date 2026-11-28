"""female_warp.py: derive a FEMALE worn model from a male one, by the displacement real items use.

usage: female_warp.py derive MALE.bin SLOT OUT.bin [--no-snap] [--k N]
       female_warp.py check [SLOT ...] [--n N]      leave-one-item-out test on real male/female pairs
       female_warp.py pairs                          rebuild data/warp-pairs.pkl and print what it holds
SLOT   head | body | legs | hands | feet | cape | weapon | shield

What it learns from (all real, read by itemlib from the cache):
  * every real item of the slot whose female worn model has the SAME topology as its male one (same vertex and face
    counts, identical face index lists): each vertex is then a pair (male position -> female position). Wear
    offsets are applied to both first. measure_items.py counts them: head 245 + moved copies, legs 98, feet 93,
    hands 16, body 8, cape 62.
  * items whose female model is a re-ordered rebuild with the same vertex count: vertices are paired by mutual
    nearest neighbour AFTER a first-pass warp, same label, within 3 units (used for body and hands, where the
    first kind is scarce; `pairs` prints how many pairs each source gave).
  * the body kits of the two standard figures, paired label by label (the smallest total distance after removing
    the label's centroid shift), and the seam table below.
The warp: a vertex takes the weighted mean displacement of the K nearest male landmarks OF THE SAME LIMB CLASS
(core, right arm, left arm, right leg, left leg; from its own label), weights falling off with distance. The
result is rounded to whole units. Then SEAM SNAP: a vertex standing exactly on a male seam-ring coordinate is put on
the female ring's coordinate and takes the female label convention (table SEAMS, measured over every distinct
real worn mesh, data/seams-frequency.txt).
weapon / shield: no warp. Real items use the same model with a female wear offset, or a copy moved by a constant;
`derive` writes the moved copy with the modal shift (WEAPON_SHIFT, SHIELD_SHIFT below).
Coordinates inside this file: cache (x, y down, z). Printed coordinates are (x, height, z).
"""
import sys, pathlib; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import json, pickle, collections
import numpy as np
from scipy.spatial import cKDTree
from scipy.optimize import linear_sum_assignment
import itemlib as L

SLOTS = dict(head=0, cape=1, weapon=3, body=4, shield=5, legs=7, hands=9, feet=10)
# male seam coordinate (x, height, z) -> female coordinate, and the label the female convention gives it per slot
# (None = keep ours). Source: data/seams-frequency.txt (share of distinct real meshes carrying each point).
SEAMS = {
    # collar: male 2 on head and body; female head 49 front / 2 back, female body 8 front / 94 back
    (-7, 164, -8): ((-4, 161, -3), {'head': 49, 'body': 8}), (7, 164, -8): ((4, 161, -3), {'head': 49, 'body': 8}),
    (-7, 165, 4): ((-5, 162, 5), {'head': 2, 'body': 94}), (7, 165, 4): ((5, 162, 5), {'head': 2, 'body': 94}),
    # neck (label 3)
    (-3, 169, -9): ((-1, 168, -5), {}), (3, 169, -9): ((1, 168, -5), {}),
    (-5, 173, 2): ((-3, 169, 3), {}), (5, 173, 2): ((3, 169, 3), {}),
    # wrist (19 / 22 both)
    (-32, 114, -2): ((-26, 110, 0), {}), (-25, 114, -2): ((-21, 110, 0), {}), (-25, 114, 4): ((-21, 110, 4), {}), (-32, 114, 4): ((-26, 110, 4), {}),
    (32, 114, -2): ((26, 110, 0), {}), (25, 114, -2): ((21, 110, 0), {}), (25, 114, 4): ((21, 110, 4), {}), (32, 114, 4): ((26, 110, 4), {}),
    # waist: male back (+-13, 114, 8) label 5; front (+-13, 114, -8) on legs (30), (+-11, 114, -8) on the body kit (4).
    # female ring (-13, 121, 5), (12, 121, 5), (-10, 121, -8), (9, 121, -8): body 4, legs 30. Not symmetric; that is the cache.
    (-13, 114, 8): ((-13, 121, 5), {'body': 4, 'legs': 30}), (13, 114, 8): ((12, 121, 5), {'body': 4, 'legs': 30}),
    (-13, 114, -8): ((-10, 121, -8), {'body': 4, 'legs': 30}), (13, 114, -8): ((9, 121, -8), {'body': 4, 'legs': 30}),
    (-11, 114, -8): ((-10, 121, -8), {'body': 4}), (11, 114, -8): ((9, 121, -8), {'body': 4}),
    # hip ring (label 29)
    (-13, 110, 8): ((-13, 117, 9), {}), (13, 110, 8): ((12, 117, 9), {}), (-13, 110, -8): ((-10, 117, -10), {}), (13, 110, -8): ((9, 117, -10), {}),
    # ankle (32 / 38 both). Female right foot ring x -11..-6, left 7..12.
    (-12, 18, -4): ((-11, 17, -3), {}), (-6, 18, -4): ((-6, 17, -3), {}), (-12, 19, 6): ((-11, 18, 3), {}), (-6, 19, 6): ((-6, 18, 3), {}),
    (12, 18, -4): ((12, 17, -3), {}), (6, 18, -4): ((7, 17, -3), {}), (12, 19, 6): ((12, 18, 3), {}), (6, 19, 6): ((7, 18, 3), {}),
    # root triangle under the feet (label 0)
    (-6, -6, -2): ((-6, -6, -2), {}), (6, -6, -2): ((6, -6, -2), {}), (0, -6, 6): ((0, -6, 3), {}),
}
WEAPON_SHIFT = (4, -5, 1)      # (x, height, z): toward the body, down, back; the modal moved copy among real weapons
SHIELD_SHIFT = (-5, -7, 1)
R_ARM = {17, 18, 19, 20, 21, 67, 27}; L_ARM = {22, 23, 24, 25, 26, 68, 28}
R_LEG = {42, 44, 34, 31, 32, 33, 80, 83, 77, 166, 48, 45}; L_LEG = {40, 43, 35, 37, 38, 36, 81, 82, 76, 167, 47, 46}
KITS = dict(m=dict(hair=230, jaw=249, body=28515, arms=26632, hands=176, legs=28285, feet=181),
            f=dict(hair=403, jaw=456, body=18554, arms=3476, hands=353, legs=14423, feet=358))
KIT_OF = dict(head=['jaw'], body=['body', 'arms'], legs=['legs'], hands=['hands'], feet=['feet'], cape=[])
PKL = L.HERE / 'data' / 'warp-pairs.pkl'


def limb(label):
    return 1 if label in R_ARM else 2 if label in L_ARM else 3 if label in R_LEG else 4 if label in L_LEG else 0


def kit_pairs(part):
    a, b = L.model(KITS['m'][part]), L.model(KITS['f'][part])
    A, B, lab = [], [], []
    for l in sorted(set(a['vertex_skins'])):
        ia = [i for i, x in enumerate(a['vertex_skins']) if x == l]; ib = [i for i, x in enumerate(b['vertex_skins']) if x == l]
        if len(ia) != len(ib): continue
        Pa = a['V'][ia].astype(float); Pb = b['V'][ib].astype(float)
        cost = np.linalg.norm((Pa - Pa.mean(0))[:, None, :] - (Pb - Pb.mean(0))[None, :, :], axis=2)
        r, c = linear_sum_assignment(cost)
        A += [Pa[i] for i in r]; B += [Pb[j] for j in c]; lab += [l] * len(r)
    return np.array(A), np.array(B), lab


def gather():
    M = json.loads((L.HERE / 'data' / 'measure-items.json').read_text())
    out = {}
    for name, slot in SLOTS.items():
        if name in ('weapon', 'shield'): continue
        src = []        # (item id, male ids, A, B, labels, source)
        seen = set()
        rows = M['female_vs_male'][str(slot)]
        for r in rows:
            if r['kind'] not in ('same topology, reshaped', 'axis-scaled copy', 'moved copy'): continue
            key = (tuple(r['male']), tuple(r['female']))
            if key in seen: continue
            seen.add(key)
            a, b = L.concat(r['male']), L.concat(r['female'])
            A = a['V'] + [0, r['off'][0], 0]; B = b['V'] + [0, r['off'][1], 0]
            src.append((r['id'], tuple(r['male']), A.astype(float), B.astype(float), list(a['vertex_skins']), 'same topology'))
        for part in KIT_OF[name]:
            A, B, lab = kit_pairs(part)
            if not len(lab): continue
            for _ in range(3):                                   # the kits are the figure itself: weight them
                src.append((-1, (KITS['m'][part],), A, B, lab, 'kit'))
        sa = np.array([[k[0], -k[1], k[2]] for k in SEAMS], float); sb = np.array([[v[0][0], -v[0][1], v[0][2]] for v in SEAMS.values()], float)
        out[name] = dict(src=src, rows=rows, seam=(sa, sb))
    # second pass: re-ordered rebuilds with equal vertex counts, matched through the first-pass warp
    for name in ('body', 'hands', 'legs', 'feet', 'head'):
        W = Warp(out[name])
        seen = {(s[1]) for s in out[name]['src']}
        added = 0
        for r in out[name]['rows']:
            if r['kind'] != 'different mesh' or tuple(r['male']) in seen: continue
            a, b = L.concat(r['male']), L.concat(r['female'])
            if a is None or b is None or len(a['V']) != len(b['V']): continue
            seen.add(tuple(r['male']))
            A = (a['V'] + [0, r['off'][0], 0]).astype(float); B = (b['V'] + [0, r['off'][1], 0]).astype(float)
            la, lb = np.array(a['vertex_skins']), np.array(b['vertex_skins'])
            P = W.apply(A, la, snap=False, rounded=False)
            ta, tb = cKDTree(P), cKDTree(B)
            d1, j = tb.query(P); d2, i = ta.query(B)
            keep = [n for n in range(len(P)) if i[j[n]] == n and d1[n] <= 3.0 and limb(la[n]) == limb(lb[j[n]])]
            if len(keep) < 0.6 * len(P): continue
            out[name]['src'].append((r['id'], tuple(r['male']), A[keep], B[j[keep]], [int(x) for x in la[keep]], 'matched rebuild'))
            added += 1
        out[name]['matched_items'] = added
    for v in out.values(): v.pop('rows')
    L.save_models()
    PKL.write_bytes(pickle.dumps(out))
    return out


_data = None


def data():
    global _data
    if _data is None:
        _data = pickle.loads(PKL.read_bytes()) if PKL.exists() else gather()
    return _data


class Warp:
    def __init__(self, slot_data, exclude=(), k=12):
        self.k = k
        A, D, C = [], [], []
        for item, mids, a, b, lab, source in slot_data['src']:
            if item in exclude or mids in exclude: continue
            A.append(a); D.append(b - a); C.append(np.array([limb(l) for l in lab]))
        self.A = np.concatenate(A); self.D = np.concatenate(D); self.C = np.concatenate(C)
        self.trees = {c: (cKDTree(self.A[self.C == c]), self.D[self.C == c]) for c in set(self.C.tolist())}
        self.all = (cKDTree(self.A), self.D)

    def apply(self, V, labels, snap=True, rounded=True, slot=None):
        V = np.asarray(V, float); out = V.copy()
        for n, (p, l) in enumerate(zip(V, labels)):
            tree, D = self.trees.get(limb(int(l)), self.all)
            k = min(self.k, tree.n)
            d, i = tree.query(p, k=k)
            d = np.atleast_1d(d); i = np.atleast_1d(i)
            if d[0] > 14 and tree is not self.all[0]:            # nothing of this limb nearby: use every landmark
                tree, D = self.all
                d, i = tree.query(p, k=min(self.k, tree.n)); d = np.atleast_1d(d); i = np.atleast_1d(i)
            s = max(3.0, d[-1] * 0.6)
            w = np.exp(-(d / s) ** 2) + 1e-9
            out[n] = p + (D[i] * w[:, None]).sum(0) / w.sum()
        return np.rint(out).astype(int) if rounded else out


def snap_seams(Vm, Vf, labels, slot):
    """Vm male (cache coords, ints), Vf warped. Returns Vf with seam vertices on the female ring and the labels."""
    Vf = Vf.copy(); labels = list(labels); n = 0
    for i, v in enumerate(Vm):
        key = (int(v[0]), int(-v[1]), int(v[2]))
        if key in SEAMS:
            to, lab = SEAMS[key]
            Vf[i] = (to[0], -to[1], to[2]); n += 1
            if slot in lab: labels[i] = lab[slot]
    return Vf, labels, n


def derive(m, slot, snap=True, k=12, exclude=()):
    """m: model dict (itemlib.load_bin). Returns (V female, vertex labels, report lines)."""
    V = m['V']; lab = list(m['vertex_skins'])
    if slot in ('weapon', 'shield'):
        s = WEAPON_SHIFT if slot == 'weapon' else SHIELD_SHIFT
        return V + [s[0], -s[1], s[2]], lab, [f'{slot}: rigid shift {s} (x, height, z)']
    W = Warp(data()[slot], exclude=exclude, k=k)
    Vf = W.apply(V, lab)
    rep = []
    if snap:
        Vf, lab, n = snap_seams(V, Vf, lab, slot)
        rep.append(f'{n} vertices on male seam coordinates snapped to the female rings')
    d = np.linalg.norm((Vf - V).astype(float), axis=1)
    rep.append(f'displacement: median {np.median(d):.1f}, max {d.max():.1f} units; box male {V.min(0).tolist()}..{V.max(0).tolist()} -> female {Vf.min(0).tolist()}..{Vf.max(0).tolist()} (cache x, y down, z)')
    # welded? two male vertices landing on one female coordinate would be merged by the client (Model::combine)
    uniq = len({tuple(v) for v in Vf.tolist()})
    if uniq != len(Vf): rep.append(f'WARNING {len(Vf) - uniq} vertices now share a coordinate with another (the client welds them when the figure is assembled)')
    return Vf, lab, rep


def check(slots, n_items):
    D = data()
    for slot in slots:
        src = [s for s in D[slot]['src'] if s[5] == 'same topology']
        step = max(1, len(src) // n_items)
        rows = []
        for item, mids, A, B, lab, _ in src[::step]:
            W = Warp(D[slot], exclude={item, mids})
            P = W.apply(A, lab, rounded=True)
            e = np.linalg.norm(P - B, axis=1); e0 = np.linalg.norm(A - B, axis=1)
            rows.append((item, len(A), np.median(e0), e0.max(), np.median(e), np.percentile(e, 95), e.max(), np.mean(e <= 1.5)))
        R = np.array([r[2:] for r in rows])
        print(f'{slot}: {len(rows)} real items tested, each left out of the fit (of {len(src)} same-topology pairs; '
              f'{sum(1 for s in D[slot]["src"] if s[5] == "matched rebuild")} matched rebuilds also in the fit)')
        print(f'   male model used as it is : median error {np.median(R[:, 0]):.2f}, worst vertex median {np.median(R[:, 1]):.2f} units')
        print(f'   warped                   : median error {np.median(R[:, 2]):.2f}, 95th percentile {np.median(R[:, 3]):.2f}, worst vertex median {np.median(R[:, 4]):.2f} '
              f'(worst item {R[:, 4].max():.1f}); vertices within 1.5 units {np.median(R[:, 5]) * 100:.0f}%')
        worst = sorted(rows, key=lambda r: -r[4])[:3]
        print('   worst items:', [(r[0], L.items()[r[0]]['name'], round(float(r[4]), 1)) for r in worst])


def main():
    skip = {i + 1 for i, x in enumerate(sys.argv) if x in ('--n', '--k')}
    a = [x for i, x in enumerate(sys.argv) if i and i not in skip and not x.startswith('--')]
    opt = lambda name, d=None: sys.argv[sys.argv.index(name) + 1] if name in sys.argv else d
    if not a: raise SystemExit(__doc__)
    if a[0] == 'pairs':
        D = gather()
        for slot, v in D.items():
            c = collections.Counter(); items = collections.Counter()
            for s in v['src']: c[s[5]] += len(s[2]); items[s[5]] += 1
            print(slot, {k: f'{items[k]} sources, {c[k]} vertex pairs' for k in c})
    elif a[0] == 'check':
        check(a[1:] or ['head', 'body', 'legs', 'hands', 'feet'], int(opt('--n', 30)))
    elif a[0] == 'derive':
        m = L.load_bin(a[1]); slot = a[2]
        if m.get('vertex_skins') is None: raise SystemExit('the male model carries no labels')
        Vf, lab, rep = derive(m, slot, snap='--no-snap' not in sys.argv, k=int(opt('--k', 12)))
        L.save_bin(a[3], m, V=Vf, vertex_skins=lab)
        print(f'{a[1]} -> {a[3]} ({slot})'); [print('  ' + r) for r in rep]
    else:
        raise SystemExit(__doc__)


if __name__ == '__main__':
    main()
