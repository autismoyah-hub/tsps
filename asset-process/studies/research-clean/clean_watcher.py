"""The mechanical test of CLEAN.md's rules: the repainted stone watcher cleaned BY SCRIPT ONLY (no hand sculpting).

usage: clean_watcher.py [--src DEF.json] [--out DIR] [--no-relax] [--no-paint] [--no-cut] [--no-open]
reads  studies/research-paint/repaint/watcher-2-repaint.bin / .json   (never written to)
writes studies/research-clean/watcher-clean/watcher-clean.bin / .json / clean-log.json

Steps (each one a rule of CLEAN.md that a script can apply):
 A  INSET, NOT LUMPS (rule F1/F2): every small closed lump stuck on the skin (eyes, sockets, nostrils, mouth corners, brow
    lumps, studs) is removed. Where it had its own colour, its footprint (the outline of its exposed part seen along
    the skin's normal, reduced to four corners) is CUT INTO THE SKIN: the skin faces under it are re-split so the
    footprint becomes two faces of the welded skin, flat on the surface, in the lump's colour. New vertices lie on the
    old surface and take the label of the nearest old vertex. A lump of the skin's own material is simply removed.
 B  PIECES OPEN WHERE THEY MEET THE SKIN (rule F4): faces of the remaining closed pieces that lie wholly inside the
    main skin are deleted (they were never seen; they only voted in the lighting).
 C  NO WEAK TONE WITHOUT A BORDER (rule S2): a one-colour patch under 0.4% of the skin that differs from a
    same-material neighbour by less than 8 lightness across a border that is not a hard fold takes that neighbour's tone.
 D  RELAX WHAT IS NOT A DELIBERATE FOLD (rule S3/D1): vertices that lie on no hard fold (50 degrees), no paint boundary,
    no open edge and no cut feature are moved along their normal toward the mean of their neighbours (Taubin, so the
    volume stays), at most 1.2% of the height.
Vertices are never deleted or renumbered (labels and every label's pivot stay); orphaned ones are left in the file.
"""
import sys, pathlib, json, math, collections, itertools
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import numpy as np
import cleanlib as cl, features as ft, cleancheck as cc
from explore import same_mat
import meshio
from shapely.geometry import Polygon, Point, LineString
from shapely.ops import unary_union
from scipy.spatial import ConvexHull

a = sys.argv[1:]
SRC = pathlib.Path(a[a.index('--src') + 1]) if '--src' in a else cl.B / 'research-paint' / 'repaint' / 'watcher-2-repaint.json'
OUT = pathlib.Path(a[a.index('--out') + 1]) if '--out' in a else HERE / 'watcher-clean'
OUT.mkdir(parents=True, exist_ok=True)
jdef = json.loads(SRC.read_text())
m = meshio.load(SRC.parent / jdef['models'][0])
AMB, CON = int(jdef.get('ambient', 0)), int(jdef.get('contrast', 0))
log = dict(source=str(SRC), steps=[])
def say(s): print(s); log['steps'].append(s)
def analyse(mm, name='work'): return cl.from_mesh(mm, name, AMB, CON)

# ------------------------------------------------------------------ working copy: faces as lists, never renumber vertices
V = [tuple(v) for v in m['vertices']]; L = list(m['vertex_skins'])
F = [tuple(f) for f in m['faces']]
ATTR = [dict(c=m['colors'][i], p=m['priorities'][i], t=m['render_types'][i], al=m['alpha'][i], d=m['depth'][i], fs=m['face_skins'][i]) for i in range(len(F))]
alive = [True] * len(F)
fixed_vertices = set()

def mesh_now():
    idx = [i for i in range(len(F)) if alive[i]]
    return dict(vertices=list(V), faces=[F[i] for i in idx], colors=[ATTR[i]['c'] for i in idx], priorities=[ATTR[i]['p'] for i in idx], render_types=[ATTR[i]['t'] for i in idx],
                alpha=[ATTR[i]['al'] for i in idx], depth=[ATTR[i]['d'] for i in idx], vertex_skins=list(L), face_skins=[ATTR[i]['fs'] for i in idx]), idx

c0 = analyse(m, 'before'); m0, off0 = cc.measure(c0)
SIGN = c0['sign']
feats, HGT = ft.find(c0)
shells = [f for f in feats if f['kind'] == 'shell']
lumps = [f for f in shells if f['closed'] and f['touching'] and f['size_h'] <= 0.07]
say(f'before: {len(F)} faces, {len(V)} vertices; {len(shells)} separate pieces, {len(lumps)} small closed lumps; splotch {m0["splotch"]:.3f}, hard/firm {m0["hard_over_firm"]:.2f}')

# main skin (largest shell) for inside tests
sizes = collections.Counter(c0['shell'][int(i)] for i in np.nonzero(c0['drawn'])[0]); MAIN = max(sizes, key=sizes.get)
main_faces = np.array([i for i in np.nonzero(c0['drawn'])[0] if c0['shell'][int(i)] == MAIN])
Vn = np.array(V, float)
MA, MB, MC = Vn[c0['F'][main_faces, 0]], Vn[c0['F'][main_faces, 1]], Vn[c0['F'][main_faces, 2]]

def tri_normal(f):
    p = np.array([V[f[0]], V[f[1]], V[f[2]]], float); n = np.cross(p[1] - p[0], p[2] - p[0]); l = np.linalg.norm(n)
    return (n / l if l > 0 else n) * SIGN

def cross2(o, p, q): return (p[0] - o[0]) * (q[1] - o[1]) - (p[1] - o[1]) * (q[0] - o[0])
def seg_cross(p1, p2, p3, p4):
    d1, d2, d3, d4 = cross2(p3, p4, p1), cross2(p3, p4, p2), cross2(p1, p2, p3), cross2(p1, p2, p4)
    return (d1 * d2 < -1e-12) and (d3 * d4 < -1e-12)
def in_tri(p, t0, t1, t2, eps=1e-9):
    d1, d2, d3 = cross2(t0, t1, p), cross2(t1, t2, p), cross2(t2, t0, p)
    return (d1 > eps and d2 > eps and d3 > eps) or (d1 < -eps and d2 < -eps and d3 < -eps)

def earclip(poly, P2):
    """poly: list of point keys (may repeat); P2[key] -> 2D. Polygon counter-clockwise. Returns triangles of keys."""
    idx = list(poly); tris = []
    guard = 0
    while len(idx) > 3 and guard < 5000:
        guard += 1; n = len(idx); best = None
        for i in range(n):
            a_, b_, c_ = idx[i - 1], idx[i], idx[(i + 1) % n]
            pa, pb, pc = P2[a_], P2[b_], P2[c_]
            ar = cross2(pa, pb, pc)
            if ar <= 1e-9: continue
            ok = True
            for k in idx:
                if k in (a_, b_, c_): continue
                pk = P2[k]
                if (pk == pa).all() or (pk == pb).all() or (pk == pc).all(): continue
                if in_tri(pk, pa, pb, pc): ok = False; break
            if ok:
                # prefer the fattest ear
                e = max(np.linalg.norm(pa - pb), np.linalg.norm(pb - pc), np.linalg.norm(pc - pa)); q = ar / (e * e)
                if best is None or q > best[0]: best = (q, i)
        if best is None:      # degenerate: clip any convex corner
            best = (0, max(range(n), key=lambda i: cross2(P2[idx[i - 1]], P2[idx[i]], P2[idx[(i + 1) % n]])))
        i = best[1]; tris.append((idx[i - 1], idx[i], idx[(i + 1) % n])); del idx[i]
    if len(idx) == 3: tris.append(tuple(idx))
    return [t for t in tris if abs(cross2(P2[t[0]], P2[t[1]], P2[t[2]])) > 1e-6]

def quad_from(points2, k=4):
    """the k-gon of largest area whose corners are hull points (counter-clockwise)"""
    if len(points2) < 3: return None
    try: hull = points2[ConvexHull(points2).vertices]
    except Exception: return None
    if len(hull) <= k: return hull
    if len(hull) > 14: hull = hull[np.round(np.linspace(0, len(hull) - 1, 14)).astype(int)]
    best = None
    for comb in itertools.combinations(range(len(hull)), k):
        p = hull[list(comb)]; ar = 0.5 * abs(sum(p[i - 1][0] * p[i][1] - p[i][0] * p[i - 1][1] for i in range(k)))
        if best is None or ar > best[0]: best = (ar, p)
    return best[1]

def cut_feature(ex, colour, skip_faces, shrinks=(1.0, 0.85, 0.7, 0.55)):
    nV = len(V)
    ok, msg = _cut(np.asarray(ex, float), colour, skip_faces, shrinks)
    if not ok: del V[nV:]; del L[nV:]
    return ok, msg

def _cut(ex, colour, skip_faces, shrinks):
    """Cut the footprint of a removed lump into the live skin as two faces of `colour`. Returns True on success."""
    Pl = ex; ctr = Pl.mean(0); ext = float(np.ptp(Pl, axis=0).max())
    live = [i for i in range(len(F)) if alive[i] and i not in skip_faces]
    cen = np.array([np.mean([V[v] for v in F[i]], axis=0) for i in live])
    near = [live[k] for k in np.nonzero(np.linalg.norm(cen - ctr, axis=1) <= 1.3 * ext + 6)[0]]
    if len(near) < 2: return False, 'no skin near'
    ns = np.array([tri_normal(F[i]) for i in near]); n = ns.mean(0)
    # the lump is outside the skin: orient by the lump's exposed side
    near_c = np.array([np.mean([V[v] for v in F[i]], axis=0) for i in near]).mean(0)
    if np.linalg.norm(n) < 0.2: n = ex.mean(0) - near_c
    n /= max(np.linalg.norm(n), 1e-9)
    u = np.cross(n, [0, 1, 0]);
    if np.linalg.norm(u) < 0.1: u = np.cross(n, [1, 0, 0])
    u /= np.linalg.norm(u); w = np.cross(n, u)
    to2 = lambda p: np.stack([(np.asarray(p, float) - ctr) @ u, (np.asarray(p, float) - ctr) @ w], -1)
    quad = quad_from(to2(ex))
    if quad is None: return False, 'no outline'
    for shrink in shrinks:
        q = quad.mean(0) + (quad - quad.mean(0)) * shrink
        P = Polygon(q)
        if not P.is_valid or P.area < 6: return False, 'footprint too small'
        cand = [i for i in near if tri_normal(F[i]) @ n > 0.2 and abs((np.mean([V[v] for v in F[i]], axis=0) - ctr) @ n) <= ext + 8]
        t2 = {i: to2([V[v] for v in F[i]]) for i in cand}
        polys = {i: Polygon(t2[i]) for i in cand}
        R = [i for i in cand if polys[i].is_valid and polys[i].intersection(P).area > 1e-6]
        for grow in range(4):
            if not R: break
            U = unary_union([polys[i] for i in R])
            ok = U.geom_type == 'Polygon' and len(U.interiors) == 0 and U.contains(P) and U.exterior.distance(P) > 0.6 and abs(sum(polys[i].area for i in R) - U.area) < 0.02 * U.area
            if ok: break
            vs = {v for i in R for v in F[i]}
            R = sorted(set(R) | {i for i in cand if polys[i].is_valid and any(v in vs for v in F[i])})
        else: ok = False
        if R and ok: break
    else: return False, 'skin under it is folded or too small'
    # boundary loop of R
    ecount = collections.Counter()
    for i in R:
        f = F[i]
        for x, y in ((f[0], f[1]), (f[1], f[2]), (f[2], f[0])): ecount[(min(x, y), max(x, y))] += 1
    nxt = collections.defaultdict(list)
    for (x, y), k in ecount.items():
        if k == 1: nxt[x].append(y); nxt[y].append(x)
    if any(len(v) != 2 for v in nxt.values()): return False, 'region border is not one loop'
    start = next(iter(nxt)); loop = [start]; prev = None
    while True:
        o = [x for x in nxt[loop[-1]] if x != prev]
        prev = loop[-1]; nx = o[0]
        if nx == start: break
        loop.append(nx)
        if len(loop) > len(nxt) + 1: return False, 'loop walk failed'
    if len(loop) != len(nxt): return False, 'region border is several loops'
    P2 = {v: to2(V[v]) for v in loop}
    if sum(cross2((0, 0), P2[loop[i - 1]], P2[loop[i]]) for i in range(len(loop))) < 0: loop.reverse()
    # footprint corners as new vertices on the old surface
    def locate(p):
        for i in R:
            t = t2[i]; d = cross2(t[0], t[1], t[2])
            if abs(d) < 1e-9: continue
            b0 = cross2(p, t[1], t[2]) / d; b1 = cross2(t[0], p, t[2]) / d; b2 = 1 - b0 - b1
            if min(b0, b1, b2) >= -1e-7: return i, (b0, b1, b2)
        return None, None
    if cross2(q[0], q[1], q[2]) < 0: q = q[::-1]
    new_ids = []
    for p in q:
        i, b = locate(p)
        if i is None: return False, 'corner off the skin'
        p3 = sum(b[k] * np.array(V[F[i][k]], float) for k in range(3))
        V.append(tuple(int(round(x)) for x in p3)); L.append(L[F[i][int(np.argmax(b))]]); new_ids.append(len(V) - 1)
        P2[len(V) - 1] = to2(V[-1])
    if len({V[i] for i in new_ids}) < len(new_ids): return False, 'corners collapse on the integer grid'
    if Polygon([P2[i] for i in new_ids]).area < 4: return False, 'footprint collapses on the integer grid'
    if cross2(P2[new_ids[0]], P2[new_ids[1]], P2[new_ids[2]]) < 0: new_ids.reverse()
    # bridge the outer loop to the hole, then clip ears
    segs = [(P2[loop[i - 1]], P2[loop[i]]) for i in range(len(loop))] + [(P2[new_ids[i - 1]], P2[new_ids[i]]) for i in range(len(new_ids))]
    pairs = sorted(((np.linalg.norm(P2[o] - P2[h]), io, ih) for io, o in enumerate(loop) for ih, h in enumerate(new_ids)))
    bridge = next(((io, ih) for _, io, ih in pairs if not any(seg_cross(P2[loop[io]], P2[new_ids[ih]], s0, s1) for s0, s1 in segs)), None)
    if bridge is None: return False, 'no bridge'
    io, ih = bridge; hole = new_ids[::-1]; jh = hole.index(new_ids[ih])
    poly = loop[:io + 1] + hole[jh:] + hole[:jh + 1] + loop[io:]
    tris = earclip(poly, P2)
    if abs(sum(abs(cross2(P2[t[0]], P2[t[1]], P2[t[2]])) / 2 for t in tris) - (Polygon([P2[v] for v in loop]).area - Polygon([P2[v] for v in new_ids]).area)) > 0.02 * Polygon([P2[v] for v in loop]).area:
        return False, 'triangulation does not cover the region'
    inner = [(new_ids[0], new_ids[i], new_ids[i + 1]) for i in range(1, len(new_ids) - 1)]
    def orient(t):
        nn = tri_normal(t); return t if nn @ n >= 0 else (t[0], t[2], t[1])
    added = []
    for t in tris:
        cpt = (P2[t[0]] + P2[t[1]] + P2[t[2]]) / 3; i, _ = locate(cpt)
        if i is None: i = min(R, key=lambda r: np.linalg.norm(t2[r].mean(0) - cpt))
        added.append((orient(t), dict(ATTR[i])))
    base = dict(ATTR[R[0]])
    for t in inner:
        at = dict(base); at['c'] = int(colour); added.append((orient(t), at))
    if any(len({V[v] for v in t}) < 3 for t, _ in added): return False, 'degenerate face'
    for i in R: alive[i] = False
    for t, at in added: F.append(t); ATTR.append(at); alive.append(True)
    fixed_vertices.update(new_ids)
    return True, f'{len(R)} skin faces re-split into {len(tris)} + {len(inner)} feature faces'

# ---------------------------------------------------------------------------------------------- A: lumps -> insets
if '--no-cut' not in a:
    done = failed = removed_only = 0
    order = sorted(lumps, key=lambda f: -f['area'])
    lump_faces = {int(i) for f in order for i in f['fs']}
    info = []
    for f in order:
        vs = sorted({int(v) for i in f['fs'] for v in c0['F'][int(i)]})
        P = Vn[vs]; ins = ft.inside_mesh(P, MA, MB, MC)
        col = collections.Counter(int(c0['C'][int(i)]) for i in f['fs']).most_common(1)[0][0]
        _, nearest = ft.tri_dist(P, MA, MB, MC); under = np.unique(main_faces[nearest])
        skin_l = float(((c0['C'][under] & 127) * c0['area'][under]).sum() / c0['area'][under].sum())
        ex = P[~ins] if (~ins).sum() >= 3 else P
        info.append(dict(f=f, ex=ex, col=col, plain=bool(f['surround_same'] and abs((col & 127) - skin_l) < 8),
                         where=f"at x {f['cen'][0]:.0f} up {f['up']:.2f} z {f['cen'][2]:.0f}, colour {cl.hsl(col)}, {f['faces']} faces, {100 * f['relief_h']:.1f}% of height proud"))
        for i in f['fs']: alive[int(i)] = False
    MIR = np.array([-1.0, 1.0, 1.0]); used_ = set()
    def snapshot(): return (len(V), len(F), list(alive), set(fixed_vertices))
    def restore(sn):
        del V[sn[0]:]; del L[sn[0]:]; del F[sn[1]:]; del ATTR[sn[1]:]; alive[:] = sn[2]; fixed_vertices.clear(); fixed_vertices.update(sn[3])
    for k, d in enumerate(info):
        if k in used_: continue
        used_.add(k); f = d['f']
        mate = next((j for j, e in enumerate(info) if j not in used_ and e['col'] == d['col'] and np.linalg.norm(e['f']['cen'] * MIR - f['cen']) < 6 and abs(f['cen'][0]) > 2), None)
        group = [d] + ([info[mate]] if mate is not None else [])
        if mate is not None: used_.add(mate)
        if d['plain']:
            for e in group: removed_only += 1; say(f"A lump {e['where']}: skin-coloured lump removed (no footprint painted)")
            continue
        okpair = False
        if mate is not None:
            both = np.concatenate([d['ex'], info[mate]['ex'] * MIR])
            for sh in (1.0, 0.85, 0.7, 0.55, 0.45):
                sn = snapshot()
                o1, m1 = cut_feature(both, d['col'], lump_faces, (sh,))
                o2, m2 = cut_feature(both * MIR, d['col'], lump_faces, (sh,)) if o1 else (False, '')
                if o1 and o2:
                    okpair = True; done += 2
                    say(f"A lump pair {d['where']} and its mirror: INSET as MIRRORED footprints (footprint scale {sh}); {m1}; {m2}"); break
                restore(sn)
        if okpair: continue
        for e in group:
            ok, msg = cut_feature(e['ex'], e['col'], lump_faces)
            if ok: done += 1; say(f"A lump {e['where']}: INSET (not mirrored), {msg}")
            else:
                failed += 1; say(f"A lump {e['where']}: could NOT be cut ({msg}); kept as it was")
                for i in e['f']['fs']: alive[int(i)] = True
    say(f'A: {done} lumps replaced by inset faces of the skin, {removed_only} skin-coloured lumps removed, {failed} kept')

# ---------------------------------------------------------------------------------------------- B: open the hidden caps
if '--no-open' not in a:
    mm, idx = mesh_now(); c1 = analyse(mm); cl.topology(c1)
    sizes1 = collections.Counter(c1['shell'][int(i)] for i in np.nonzero(c1['drawn'])[0]); main1 = max(sizes1, key=sizes1.get)
    mf = np.array([i for i in np.nonzero(c1['drawn'])[0] if c1['shell'][int(i)] == main1])
    A1, B1, C1 = c1['V'][c1['F'][mf, 0]], c1['V'][c1['F'][mf, 1]], c1['V'][c1['F'][mf, 2]]
    others = [i for i in np.nonzero(c1['drawn'])[0] if c1['shell'][int(i)] != main1]
    ov = np.unique(c1['F'][others]) if others else np.zeros(0, int)
    insv = dict(zip(ov.tolist(), ft.inside_mesh(c1['V'][ov], A1, B1, C1).tolist()))
    cen_in = ft.inside_mesh(c1['cen'][others], A1, B1, C1) if others else []
    dist, _ = ft.tri_dist(c1['cen'][others], A1, B1, C1) if others else (np.zeros(0), None)
    gone = 0
    for k, i in enumerate(others):
        if cen_in[k] and all(insv[int(v)] for v in c1['F'][i]) and dist[k] > 2.0:
            alive[idx[i]] = False; gone += 1
    say(f'B: {gone} hidden faces of separate pieces (wholly inside the main skin) deleted: closed pieces become open where they meet the skin')

# ---------------------------------------------------------------------------------------------- C: weak tone patches
def absorb_weak(max_share=0.004, weak=8, passes=8):
    changed = 0
    for _ in range(passes):
        mm, idx = mesh_now(); c = analyse(mm); cc.prep(c)
        parent = {int(i): int(i) for i in np.nonzero(c['skinmask'])[0]}
        def find(x):
            while parent[x] != x: parent[x] = parent[parent[x]]; x = parent[x]
            return x
        for x, y, s in zip(c['sea'], c['seb'], c['samecol']):
            if s: parent[find(int(x))] = find(int(y))
        pa = collections.Counter(); members = collections.defaultdict(list)
        for i in parent: r = find(i); pa[r] += c['area'][i]; members[r].append(i)
        nb = collections.defaultdict(lambda: collections.Counter()); border = collections.Counter()
        pl = (c['C'] & 127)
        for x, y, l, s, d in zip(c['sea'], c['seb'], c['sel'], c['sm'], c['sdih']):
            rx, ry = find(int(x)), find(int(y))
            if rx == ry: continue
            border[rx] += l; border[ry] += l
            if s and abs(d) < cc.HARD and abs(int(pl[x]) - int(pl[y])) < weak:
                nb[rx][ry] += l; nb[ry][rx] += l
        n = 0; recol = {}
        for r in sorted(pa, key=lambda r: pa[r]):
            if pa[r] > max_share * c['tot'] or not nb[r] or r in recol: continue
            if sum(nb[r].values()) < 0.5 * border[r]: continue          # mostly bordered by hard folds or strong steps: a facet or a mark, keep
            tgt, _ = max(nb[r].items(), key=lambda kv: (kv[1], pa[kv[0]]))
            if pa[tgt] < pa[r] or tgt in recol: continue
            recol[r] = int(c['C'][members[tgt][0]]); n += 1
        for r, col in recol.items():
            for i in members[r]: ATTR[idx[i]]['c'] = col
        changed += n
        if n == 0: break
    return changed
if '--no-paint' not in a:
    n = absorb_weak(); say(f'C: {n} weak one-colour patches (under 0.4% of the skin, under 8 lightness from a same-material neighbour, no hard fold between) took their neighbour\'s tone')

# ---------------------------------------------------------------------------------------------- D: relax
def relax(iters=12, lam=0.5, mu=-0.53, cap_share=0.012):
    mm, idx = mesh_now(); c = analyse(mm); cc.prep(c); cl.topology(c)
    used = set(np.unique(c['F'][c['drawn']]).tolist())
    fixed = set(fixed_vertices)
    for (x, y), d, sc in zip(c['sev'], c['sdih'], c['samecol']):
        if abs(d) >= cc.HARD or not sc: fixed.add(int(x)); fixed.add(int(y))
    for (x, y) in c['open']: fixed.add(int(x)); fixed.add(int(y))
    # translucent or undrawn faces: leave their vertices alone
    for i in np.nonzero(~c['skinmask'])[0]:
        for v in c['F'][i]: fixed.add(int(v))
    # nothing moves where two pieces meet: a vertex within 2.5% of the height of another shell stays
    dr = np.nonzero(c['drawn'])[0]; sh = np.array([c['shell'][int(i)] for i in dr]); vshell = {}
    for i, s_ in zip(dr, sh):
        for v in c['F'][i]: vshell[int(v)] = s_
    for s_ in set(sh.tolist()):
        mine = [v for v in used if vshell.get(v) == s_ and v not in fixed]; of = dr[sh != s_]
        if not mine or not len(of): continue
        dist, _ = ft.tri_dist(c['V'][mine], c['V'][c['F'][of, 0]], c['V'][c['F'][of, 1]], c['V'][c['F'][of, 2]])
        fixed.update(v for v, d_ in zip(mine, dist) if d_ < 0.025 * c['hgt'])
    free = sorted(v for v in used if v not in fixed and len(c['vv'][v]) >= 3)
    P = np.array(V, float); P0 = P.copy(); cap = cap_share * c['hgt']
    nbrs = {v: np.array(sorted(c['vv'][v])) for v in free}
    # vertex normals from the starting mesh (area weighted)
    vn = np.zeros_like(P)
    for i in np.nonzero(c['drawn'])[0]:
        for v in c['F'][i]: vn[v] += c['unit'][i] * c['area'][i]
    vn /= np.maximum(np.linalg.norm(vn, axis=1), 1e-9)[:, None]
    for it in range(iters):
        for f in (lam, mu):
            delta = np.zeros_like(P)
            for v in free:
                d = P[nbrs[v]].mean(0) - P[v]; delta[v] = vn[v] * (d @ vn[v])
            P += f * delta
            off = P - P0; ln = np.linalg.norm(off, axis=1); big = ln > cap
            P[big] = P0[big] + off[big] * (cap / ln[big])[:, None]
    moved = 0; mx = 0.0
    for v in free:
        nv = tuple(int(round(x)) for x in P[v])
        if nv != V[v]: moved += 1; mx = max(mx, float(np.linalg.norm(np.array(nv) - np.array(V[v])))); V[v] = nv
    return moved, len(free), mx
if '--no-relax' not in a:
    mv, fr, mx = relax(); say(f'D: {mv} of {fr} free vertices moved along their normals (largest move {mx:.1f} units = {100 * mx / HGT:.2f}% of the height); vertices on hard folds, paint boundaries, open edges and cut features did not move')
    if '--no-paint' not in a:
        n = absorb_weak(); say(f'C again after relaxing: {n} more weak patches absorbed')

# ---------------------------------------------------------------------------------------------- write and check
mm, idx = mesh_now()
bad = [f for f in mm['faces'] if len({mm['vertices'][v] for v in f}) < 3]
if bad: say(f'{len(bad)} faces degenerate after rounding: removed'); keep = [i for i, f in enumerate(mm['faces']) if len({mm['vertices'][v] for v in f}) == 3]; mm = {k: ([v[i] for i in keep] if k not in ('vertices', 'vertex_skins') else v) for k, v in mm.items()}
meshio.save(mm, OUT / 'watcher-clean.bin')
jo = dict(jdef); jo['name'] = 'Meridian Watcher (cleaned by script)'; jo['models'] = ['watcher-clean.bin']
(OUT / 'watcher-clean.json').write_text(json.dumps(jo, indent=1))
c2 = analyse(mm, 'after'); m2, _ = cc.measure(c2)
# labels and pivots: every label's vertex centroid before and after
L0 = np.array(m['vertex_skins']); V0 = np.array(m['vertices'], float); L2 = np.array(mm['vertex_skins']); V2 = np.array(mm['vertices'], float)
shift = {int(l): float(np.linalg.norm(V0[L0 == l].mean(0) - V2[L2 == l].mean(0))) for l in np.unique(L0)}
say(f"labels: {len(set(L0.tolist()))} before, {len(set(L2.tolist()))} after; largest move of any label's vertex centroid {max(shift.values()):.2f} units (label {max(shift, key=shift.get)}); "
    f"vertices {len(V0)} -> {len(V2)} (none removed), faces {len(m['faces'])} -> {len(mm['faces'])}")
rows = cc.bands()
log['before'] = cc.jsonable(m0); log['after'] = cc.jsonable(m2)
print(); cc.report('BEFORE: ' + jdef['name'], m0, rows); print(); cc.report('AFTER: cleaned by script', m2, rows)
(OUT / 'clean-log.json').write_text(json.dumps(log, indent=1))
