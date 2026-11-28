"""How real weighted creatures PAINT their joints: a census of every bone-to-child boundary on many creatures.

usage: ring_census.py [--json OUT] [--max-seqs 8] [--only NPC,NPC]
One creature per frame base (the largest model list with any shared vertex), read from the cache (read only).
For each bone c with parent p where both own vertices:
  boundary     mesh edges (welded) joining a vertex dominated by p to one dominated by c. None = a separate shell.
  ring k       hop distance from that boundary: +1 = the c-side vertices touching p's region, +2 the next, ..;
               -1, -2 on p's side. Per ring: the share of vertices carrying BOTH bones, the mean share of the bone
               across the joint (c-side: p's weight; p-side: c's weight), and the byte values in use.
  x            the vertex's signed distance from the joint along the limb (bisector of parent and child direction),
               in units of the limb's RADIUS at the joint (median distance of rings +-1 from the axis).
  kind         from the creature's own sequences: the child's rotation against rest over up to --max-seqs sequences:
               rigid (never over 8 degrees, or no boundary), chain (a run of 3+ single-child bones), hinge (90% of
               the rotation about one axis), ball (the rest).
Prints the table by kind; --json keeps every joint.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, json, pathlib, collections
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
import riglib as R
L, B, RM = R.L, R.B, R.RM
a = sys.argv
def opt(n, d=None): return a[a.index(n) + 1] if n in a else d
MAXS = int(opt('--max-seqs', 8))


def rotvec(Rm):
    """Rotation matrices (..., 3, 3) -> rotation vectors (axis x angle)."""
    tr = np.clip((np.trace(Rm, axis1=-2, axis2=-1) - 1) / 2, -1, 1); ang = np.arccos(tr)
    ax = np.stack([Rm[..., 2, 1] - Rm[..., 1, 2], Rm[..., 0, 2] - Rm[..., 2, 0], Rm[..., 1, 0] - Rm[..., 0, 1]], -1)
    n = np.linalg.norm(ax, axis=-1, keepdims=True); ax = np.where(n > 1e-9, ax / np.maximum(n, 1e-12), 0)
    return ax * ang[..., None]


def motion(sk, bid, maxs=MAXS):
    """Per bone: largest angle from rest (degrees) and the share of its rotation about one axis, over the base's sequences."""
    seqs = L.seqs_on_base(bid); seqs = sorted(seqs, key=lambda s: -s[1])[:maxs]; rv = []
    rest = sk.local[:, :3, :3]; rest = rest / np.linalg.norm(rest, axis=-1, keepdims=True)
    for sid, n in seqs:
        try:
            aid, _ = L.anim_id(sid); an = B.decode_anim(L.anim_bytes(aid))
        except Exception: continue
        tb = RM.channel_table(an, sk, range(0, n + 1, 3)); M = RM.compose(tb[..., 0:3], tb[..., 3:6], np.ones_like(tb[..., 6:9]))[..., :3, :3]
        rv.append(rotvec(np.swapaxes(rest, -1, -2)[None] @ M))
    if not rv: return np.zeros(sk.n), np.zeros(sk.n), 0
    rv = np.concatenate(rv); ang = np.degrees(np.linalg.norm(rv, axis=-1)).max(0); share = np.zeros(sk.n)
    for i in range(sk.n):
        x = rv[:, i] - rv[:, i].mean(0); s = np.linalg.svd(x, compute_uv=False) ** 2
        share[i] = s[0] / s.sum() if s.sum() > 1e-12 else 1.0
    # range of motion: largest distance between two rotation vectors (good under 180 degrees)
    rng = np.degrees(np.array([np.linalg.norm(rv[:, i].max(0) - rv[:, i].min(0)) for i in range(sk.n)]))
    return np.maximum(ang, rng), share, len(seqs)


def analyse(sk, m, ang, share):
    V = np.array(m['vertices'], float); F = np.array(m['faces']); W = m['bone_weights']; nv = len(V)
    Wd = RM.weight_matrix(W, sk.n); dom = Wd.argmax(1)
    _, inv = np.unique(np.rint(V).astype(int), axis=0, return_inverse=True); inv = inv.ravel()
    rep = {}                                             # welded id -> first vertex
    for i, k in enumerate(inv): rep.setdefault(int(k), i)
    w2v = collections.defaultdict(list)
    for i, k in enumerate(inv): w2v[int(k)].append(i)
    nb = collections.defaultdict(set)
    for f in inv[F]:
        x, y, z = int(f[0]), int(f[1]), int(f[2]); nb[x] |= {y, z}; nb[y] |= {x, z}; nb[z] |= {x, y}
    domw = {k: int(dom[v]) for k, v in rep.items()}; H = sk.head()
    owned = collections.Counter(domw.values())
    wkids = lambda i: [c for c in sk.children[i] if any(owned.get(d, 0) for d in subtree(c))]
    memo = {}
    def subtree(b):
        if b not in memo: memo[b] = [b] + [y for c in sk.children[b] for y in subtree(c)]
        return memo[b]
    def run_len(c):
        """Length of the single-child run of weighted bones through c (up and down)."""
        n = 1; q = c
        while len(wkids(q)) == 1: q = wkids(q)[0]; n += 1
        q = c
        while sk.parent[q] >= 0 and len(wkids(sk.parent[q])) == 1 and owned.get(sk.parent[q], 0): q = sk.parent[q]; n += 1
        return n
    rows = []
    for c in range(sk.n):
        p = sk.parent[c]
        if p < 0 or not owned.get(c) or not owned.get(p): continue
        Pk = {k for k, d in domw.items() if d == p}; Ck = {k for k, d in domw.items() if d == c}
        b_c = {k for k in Ck if nb[k] & Pk}; b_p = {k for k in Pk if nb[k] & Ck}
        row = dict(bone=c, parent=p, verts_c=len(Ck), verts_p=len(Pk), boundary=len(b_c) + len(b_p), angle=float(ang[c]), axis_share=float(share[c]),
                   run=run_len(c), siblings=len(wkids(p)), leaf=not wkids(c))
        if len(b_c) + len(b_p) < 3:
            row['kind'] = 'rigid-separate'; rows.append(row); continue
        # hop rings
        ring = {}
        for side, start, region in ((1, b_c, Ck), (-1, b_p, Pk)):
            cur = set(start); k = 1; seen = set(start)
            while cur and k <= 5:
                for v in cur: ring[v] = side * k
                nxt = set()
                for v in cur: nxt |= (nb[v] & region) - seen
                seen |= nxt; cur = nxt; k += 1
        # axis and radius
        kid = max(wkids(c), key=lambda q: len(subtree(q))) if wkids(c) else None
        ap = H[c] - H[p]; cen = V[[rep[k] for k in Ck]].mean(0)
        ac = (H[kid] - H[c]) if kid is not None and np.linalg.norm(H[kid] - H[c]) > 1 else (cen - H[c])
        ap = ap / max(np.linalg.norm(ap), 1e-9); ac = ac / max(np.linalg.norm(ac), 1e-9)
        ax = ap + ac if np.linalg.norm(ap) > .5 and ap @ ac > -0.5 else ac
        ax = ax / max(np.linalg.norm(ax), 1e-9)
        bv = V[[rep[k] for k in (b_c | b_p)]] - H[c]; rad = float(np.median(np.linalg.norm(bv - np.outer(bv @ ax, ax), axis=1)))
        row['radius'] = rad; row['bone_len'] = float(np.linalg.norm(H[c] - H[p])); row['bend_rest'] = float(np.degrees(np.arccos(np.clip(ap @ ac, -1, 1))))
        per = {}; pts = []
        for k, r in ring.items():
            v = rep[k]; wp, wc = Wd[v, p], Wd[v, c]; other = wp if r > 0 else wc; two = wp > 0 and wc > 0
            third = 1 - wp - wc
            d = per.setdefault(r, dict(n=0, two=0, other=0.0, third=0.0, vals=collections.Counter()))
            d['n'] += 1; d['two'] += int(two); d['other'] += other; d['third'] += third
            if two: d['vals'][int(round(other * 255))] += 1
            dv = V[v] - H[c]; x = float(dv @ ax) / max(rad, 1e-6); lat = float(np.linalg.norm(dv - (dv @ ax) * ax)) / max(rad, 1e-6)
            if wp + wc > 0.85: pts.append((x, wc / (wp + wc), r, lat))
        row['rings'] = {int(r): dict(n=d['n'], two=d['two'] / d['n'], other=d['other'] / d['n'], third=d['third'] / d['n'], vals=d['vals'].most_common(3)) for r, d in sorted(per.items())}
        row['pts'] = [(round(x, 2), round(f, 3), r, round(lat, 2)) for x, f, r, lat in pts]
        row['len_c'] = float(np.linalg.norm(H[kid] - H[c])) if kid is not None else float(2 * np.linalg.norm(cen - H[c]))
        bx = np.array([float((V[rep[k]] - H[c]) @ ax) for k in (b_c | b_p)]) / max(rad, 1e-6); row['bnd_x_med'] = float(np.median(bx)); row['bnd_x_iqr'] = float(np.quantile(bx, .75) - np.quantile(bx, .25))
        # does the boundary go round the limb (a tube's ring) or lie to one side of the axis (a plate, a membrane's edge)?
        perp = bv - np.outer(bv @ ax, ax); e1 = np.cross(ax, [1.0, 0, 0]) if abs(ax[0]) < .9 else np.cross(ax, [0, 1.0, 0]); e1 /= np.linalg.norm(e1); e2 = np.cross(ax, e1)
        th = np.sort(np.arctan2(perp @ e2, perp @ e1)); gaps = np.diff(np.concatenate([th, th[:1] + 2 * np.pi])); row['gap'] = float(np.degrees(gaps.max()))
        cv = V[[rep[k] for k in Ck]]; sv = np.linalg.svd(cv - cv.mean(0), compute_uv=False) if len(cv) >= 4 else np.array([1.0, 1.0, 1.0]); row['flat_c'] = float(sv[2] / max(sv[0], 1e-9)); row['thin_c'] = float(sv[1] / max(sv[0], 1e-9))
        row['off_axis'] = float(np.linalg.norm(perp.mean(0)) / max(rad, 1e-6))
        row['rings_shared_c'] = sum(1 for r, d in row['rings'].items() if r > 0 and d['two'] >= 0.5)
        row['rings_shared_p'] = sum(1 for r, d in row['rings'].items() if r < 0 and d['two'] >= 0.5)
        if row['angle'] < 8: row['kind'] = 'rigid-still'
        elif row['run'] >= 4 and row['siblings'] == 1: row['kind'] = 'chain'
        elif row['axis_share'] >= 0.9: row['kind'] = 'hinge'
        else: row['kind'] = 'ball'
        rows.append(row)
    return rows


def creatures():
    cen = json.loads((pathlib.Path(_AP.fix(r'<ROOT>\studies\research-rig\data\rig-census.json'))).read_text()); best = {}
    for x in cen:
        if x['one'] > 0.97 or x['vertices'] < 280: continue
        d = L.npc_def(x['npc']); bid = L.base_of_seq(d['anims'][0])
        if bid not in best or x['vertices'] > best[bid]['vertices']: best[bid] = dict(x, base=bid, models=d['models'])
    return sorted(best.values(), key=lambda x: -x['vertices'])


if __name__ == '__main__':
    todo = creatures()
    if opt('--only'): ids = {int(x) for x in opt('--only').split(',')}; todo = [x for x in todo if x['npc'] in ids]
    out = []
    for x in todo:
        sk = RM.Skeleton(B.decode_base(L.base_bytes(x['base']))); ang, share, ns = motion(sk, x['base']); rows = []
        for mid in x['models']:
            m = B.decode_model(L.model_bytes(mid))
            if m['bone_weights'] is None: continue
            for r in analyse(sk, m, ang, share): r['model'] = mid; rows.append(r)
        kinds = collections.Counter(r['kind'] for r in rows)
        print(f"{x['npc']:6d} {x['name'][:24]:24s} base {x['base']} bones {sk.n:3d} verts {x['vertices']:5d} seqs {ns}: joints {len(rows)} {dict(kinds)}", flush=True)
        out.append(dict(npc=x['npc'], name=x['name'], base=x['base'], bones=sk.n, vertices=x['vertices'], size=x['size'], one=x['one'], joints=rows))
    if opt('--json'): pathlib.Path(opt('--json')).write_text(json.dumps(out))
