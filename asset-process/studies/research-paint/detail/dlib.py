"""Small inlaid details: the real boss's marks, found on its mesh, followed into the harness's pictures, and looked
for at the same place in a rebuild's pictures. Library of detail.py and detailgate.py (research-paint/detail).

All pictures are drawn by the client's harness through research-tone/tl.py (LIT picture + ID pictures giving the front
face of every pixel). ALL temporary storage of tl / cmesh / rebuild_lib is redirected into ./work.

A DETAIL (mark) is a connected one-colour region of the drawn, opaque skin, at most 1.2% of the drawn surface, that
contrasts with what surrounds it:
  strict (cmesh.py's rule): EVERY neighbouring region differs by >= 12 paint lightness or lies in another hue family;
  loose  (research-tone's rule): its paint differs by >= 20 grey levels (0-255) from its LARGEST neighbour.
Cameras (px per unit before the definition's scale): game = zoom 1150, pitch 220 (0.445; four quarter yaws);
close = zoom 575, pitch 128 (0.89; two yaws); board = the blind line-up's own camera (zoom 512 x D x 1.75 / 300,
pitch 128, yaw 192: what the owner was shown).
"""
import sys, pathlib, json, collections
B = pathlib.Path(__file__).resolve().parent
BUILDS = B.parents[1]
sys.path.insert(0, str(BUILDS / 'research-tone'))
import numpy as np, cv2
import tl
from tl import L
import cmesh
WORK = B / 'work'; (WORK / 'data').mkdir(parents=True, exist_ok=True)
tl.WORK = WORK; cmesh.DATA = WORK / 'data'; L.HERE = WORK
RENDER = BUILDS.parent / 'render' / 'research-paint-detail'; RENDER.mkdir(parents=True, exist_ok=True)

OURS = {
    'tekton1': (7541, BUILDS / 'boss-rebuild-7541' / 'boss-rebuild-7541.json'),
    'tekton2': (7541, BUILDS / 'boss-rebuild-7541-2' / 'boss-rebuild-7541-2.json'),
    'tekton3': (7541, BUILDS / 'boss-rebuild-7541-3' / 'boss-rebuild-7541-3.json'),
    'nex': (11278, BUILDS / 'boss-rebuild-11278' / 'boss-rebuild-11278.json'),
    'lynx': (11068, BUILDS / 'creature-rebuild-11068' / 'creature-rebuild-11068.json'),
    'minotaur': (12812, BUILDS / 'creature-rebuild-12812' / 'creature-rebuild-12812.json'),
    'araxyte': (11175, BUILDS / 'creature-rebuild-11175' / 'creature-rebuild-11175.json'),
}
ROWS = ['bind', 'stand:0', 'walk:3', 'attack:4']
PX_MIN, C_MIN, HALF = 2, 12.0, 0.5          # a mark counts at game size from 2 pixels and 12 grey levels; kept if half its contrast is there
SIZE_PX = [(1, 1, '1 px'), (2, 3, '2-3 px'), (4, 8, '4-8 px'), (9, 20, '9-20 px'), (21, 10 ** 9, '21+ px')]
SIZE_U = [(0, 8, '<8 u'), (8, 16, '8-16 u'), (16, 32, '16-32 u'), (32, 64, '32-64 u'), (64, 10 ** 9, '64+ u')]
CON = [(0, 12, '<12'), (12, 25, '12-25'), (25, 50, '25-50'), (50, 999, '50+')]
def cls(v, table):
    for lo, hi, name in table:
        if lo <= v <= hi if table is SIZE_PX else lo <= v < hi: return name
    return table[-1][2]

# ---------------------------------------------------------------- mesh side
def g255(c): return float(tl.TABLE[int(c)] @ tl.GREY)
def mesh_marks(a, scale=128):
    """Every one-colour region of the drawn skin; returns (root per face, list of marks). A mark is a dict:
    faces, area (units^2 after scale), long / wide (units after scale), dgrey (paint grey against the neighbours,
    signed, weighted by shared edge length), strict, loose, translucent, centre."""
    V, F, C = a['V'], a['F'], a['C']; dr = tl.drawn(a); s = scale / 128.0
    area = np.linalg.norm(np.cross(V[F[:, 1]] - V[F[:, 0]], V[F[:, 2]] - V[F[:, 0]]), axis=1) / 2
    tot = area[dr].sum(); parent = list(range(len(F)))
    def find(i):
        while parent[i] != i: parent[i] = parent[parent[i]]; i = parent[i]
        return i
    edges = collections.defaultdict(list)
    for i in np.nonzero(dr)[0]:
        f = F[i]
        for x, y in ((f[0], f[1]), (f[1], f[2]), (f[2], f[0])): edges[(min(x, y), max(x, y))].append(i)
    for v in edges.values():
        for j in v[1:]:
            if C[v[0]] == C[j] and (a['A'][v[0]] > 0) == (a['A'][j] > 0): parent[find(j)] = find(v[0])
    root = np.array([find(i) for i in range(len(F))]); ra = collections.Counter()
    for i in np.nonzero(dr)[0]: ra[root[i]] += area[i]
    neigh = collections.defaultdict(collections.Counter)
    for (x, y), v in edges.items():
        el = np.linalg.norm(V[x] - V[y])
        for i in v:
            for j in v:
                if root[i] != root[j]: neigh[root[i]][root[j]] += el
    by = collections.Counter()
    for c, ar in zip(C[dr], area[dr]): by[int(c)] += ar
    cmesh.HMAP.clear(); cmesh.HMAP.update(cmesh.hue_clusters([c for c, ar in by.items() if ar >= .004 * tot]))
    fam = {int(c): cmesh.family(int(c)) for c in by}
    faces_of = collections.defaultdict(list)
    for i in np.nonzero(dr)[0]: faces_of[root[i]].append(i)
    marks = []
    for r, ar in ra.items():
        if ar > .012 * tot or not neigh[r]: continue
        c = int(C[r]); l = c & 127; ns = neigh[r]
        strict = all(abs(l - (int(C[q]) & 127)) >= 12 or fam[c] != fam[int(C[q])] for q in ns)
        big = max(ns.items(), key=lambda kv: kv[1])[0]
        loose = abs(g255(c) - g255(C[big])) >= 20
        if not (strict or loose): continue
        w = sum(ns.values()); dg = g255(c) - sum(g255(C[q]) * e for q, e in ns.items()) / w
        fs = np.array(faces_of[r]); P = V[np.unique(F[fs])] * s; P0 = P - P.mean(0)
        if len(P) >= 3:
            u, sv, vt = np.linalg.svd(P0, full_matrices=False); long = float(np.ptp(P0 @ vt[0]))
        else: long = float(np.linalg.norm(np.ptp(P, 0)))
        A = float(ar) * s * s
        marks.append(dict(root=int(r), faces=fs, area=A, long=long, wide=A / max(long, 1e-6), dgrey=float(dg), strict=bool(strict), loose=bool(loose),
                          translucent=bool(a['A'][r] > 0), centre=P.mean(0), colour=c))
    return root, marks

# ---------------------------------------------------------------- pictures
def cameras(npc):
    lo, hi, _, sc = L.size_after_scale(npc); D = float((hi - lo).max()); aim = int((hi[1] - lo[1]) / 2)
    gc = max(140, int(D * .445 * 1.7))
    return dict(
        game=dict(zoom=1150, pitch=220, aim=aim, cell=(gc, gc), yaws=(192, 704, 1216, 1728), k=2, ppu=0.445),
        close=dict(zoom=575, pitch=128, aim=aim, cell=(2 * gc, 2 * gc), yaws=(192, 1216), k=4, ppu=0.89),
        board=dict(zoom=int(512 * D * 1.75 / 300), pitch=128, aim=aim, cell=(300, 300), yaws=(192,), k=1, ppu=300 / (D * 1.75)),
    ), sc
def facemap(i0, i1, nf):
    c0, c1 = tl.decode_codes(i0), tl.decode_codes(i1); f = np.where((c0 >= 0) & (c1 >= 0), c0 + tl.NCODE * c1, -1); f[f >= nf] = -1
    return f
def pictures(tag, npc, mesh, defn, lit_entry, seqs, rows=ROWS, cams=None, refresh=False):
    """LIT cells and the front face per pixel for one mesh under every camera. Cached in work/px/TAG.npz."""
    px = WORK / 'px'; px.mkdir(exist_ok=True); f = px / f'{tag}.npz'
    allc, _ = cameras(npc); cams = cams or list(allc)
    if f.exists() and not refresh:
        z = np.load(f); return {k: z[k] for k in z.files}
    ids = tl.id_variants(tag, mesh, defn, 'ids')
    ents = [lit_entry] + [tl.entry(p, seqs) for p in ids[:2]]
    out = {}
    for cn in cams:
        cam = allc[cn]
        for yaw in cam['yaws']:
            cells, _ = tl.draw(ents, rows, cam, yaw)
            for row, cs in zip(rows, cells):
                out[f'{cn}|{yaw}|{row}|lit'] = cs[0]; out[f'{cn}|{yaw}|{row}|face'] = facemap(cs[1], cs[2], len(mesh['faces'])).astype(np.int32)
    np.savez_compressed(f, **out)
    return out
def real_pictures(npc, rows=ROWS):
    m, kv = tl.real_mesh(npc); d = tl.real_def(npc)
    if d['recolor']: raise SystemExit('recoloured NPC: not handled')
    seqs = L.sequences(npc)
    return m, d, seqs, pictures(f'real-{npc}', npc, m, d, L.entry_real(npc, seqs), seqs, rows)
def ours_pictures(tag, npc, json_path, rows=ROWS):
    m, j = tl.ours_mesh(json_path); seqs = L.sequences(npc)
    return m, j, pictures(f'ours-{tag}', npc, m, j, tl.entry(json_path, seqs), seqs, rows)

def footprints(a, marks, face, lit, k):
    """Per mark seen in this cell: (index, footprint mask bbox-cropped data). Returns list of dicts with px, contrast
    (inside minus ring, grey 0-255), and the masks' coordinates for re-use on another picture."""
    nf = len(a['F']); of = np.full(nf + 1, -1); solidf = np.zeros(nf + 1, bool)
    for i, mk in enumerate(marks): of[mk['faces']] = i
    solidf[:nf] = (a['A'] == 0) & tl.drawn(a)
    pm = of[face]; solid = solidf[face] & (face >= 0)           # face -1 -> index nf (last): not solid
    g = tl.grey(lit); H, W = face.shape; out = []
    ker = cv2.getStructuringElement(cv2.MORPH_ELLIPSE, (2 * k + 1, 2 * k + 1))
    for i in np.unique(pm[pm >= 0]):
        fp = (pm == i) & solid
        n = int(fp.sum())
        if n == 0: continue
        ys, xs = np.nonzero(fp); y0, y1, x0, x1 = max(ys.min() - k - 1, 0), min(ys.max() + k + 2, H), max(xs.min() - k - 1, 0), min(xs.max() + k + 2, W)
        sub = fp[y0:y1, x0:x1]; ring = (cv2.dilate(sub.astype(np.uint8), ker) > 0) & ~sub & solid[y0:y1, x0:x1] & (pm[y0:y1, x0:x1] != i)
        if ring.sum() < 2: continue
        gs = g[y0:y1, x0:x1]
        out.append(dict(mark=int(i), px=n, contrast=float(gs[sub].mean() - gs[ring].mean()), box=(y0, y1, x0, x1), fp=sub, ring=ring,
                        span=int(max(np.ptp(ys), np.ptp(xs)) + 1)))
    return out
def paint_out(m, a, marks):
    """A copy of the mesh with every mark repainted in the colour of its largest neighbour that is not itself a mark
    (by shared edge length; marks surrounded by marks take a resolved neighbour's colour on a later pass)."""
    V, F, C = a['V'], a['F'], a['C']; dr = tl.drawn(a); col = C.copy()
    of = np.full(len(F), -1)
    for i, mk in enumerate(marks): of[mk['faces']] = i
    edges = collections.defaultdict(list)
    for i in np.nonzero(dr)[0]:
        f = F[i]
        for x, y in ((f[0], f[1]), (f[1], f[2]), (f[2], f[0])): edges[(min(x, y), max(x, y))].append(i)
    nb = [collections.Counter() for _ in marks]            # neighbour face -> shared length
    for (x, y), v in edges.items():
        el = float(np.linalg.norm(V[x] - V[y]))
        for i in v:
            for j in v:
                if of[i] >= 0 and of[j] != of[i]: nb[of[i]][j] += el
    done = np.zeros(len(marks), bool)
    for _ in range(8):
        newly = []
        for i, mk in enumerate(marks):
            if done[i]: continue
            c = collections.Counter()
            for j, el in nb[i].items():
                if of[j] < 0 or done[of[j]]: c[int(col[j])] += el
            if c: newly.append((i, c.most_common(1)[0][0]))
        if not newly: break
        for i, cc in newly: col[marks[i]['faces']] = cc; done[i] = True
    mm = tl.clone(m); mm['colors'] = [int(c) for c in col]
    return mm, int(done.sum())
def lit_only(tag, npc, mesh, defn, seqs, rows=ROWS):
    """LIT cells of a variant mesh under every camera (cached)."""
    px = WORK / 'px'; px.mkdir(exist_ok=True); f = px / f'{tag}.npz'
    if f.exists():
        z = np.load(f); return {k: z[k] for k in z.files}
    p = tl.write_variant(tag, mesh, defn, 'variants'); allc, _ = cameras(npc); out = {}
    for cn, cam in allc.items():
        for yaw in cam['yaws']:
            cells, _ = tl.draw([tl.entry(p, seqs)], rows, cam, yaw)
            for row, cs in zip(rows, cells): out[f'{cn}|{yaw}|{row}|lit'] = cs[0]
    np.savez_compressed(f, **out)
    return out

def instances(a, marks, face, lit, lit0, k):
    """Per mark seen in this cell: footprint, ring (nearby solid pixels on no mark), its EFFECT on the picture
    (real minus the same mesh with every mark painted out, RGB) and the old inside-minus-ring grey contrast."""
    nf = len(a['F']); of = np.full(nf + 1, -1); solidf = np.zeros(nf + 1, bool)
    for i, mk in enumerate(marks): of[mk['faces']] = i
    solidf[:nf] = (a['A'] == 0) & tl.drawn(a)
    pm = of[face]; solid = solidf[face] & (face >= 0)
    g = tl.grey(lit); H, W = face.shape; out = []
    D = lit.astype(float) - lit0.astype(float)
    ker = cv2.getStructuringElement(cv2.MORPH_ELLIPSE, (2 * k + 1, 2 * k + 1))
    for i in np.unique(pm[pm >= 0]):
        fp = (pm == i) & solid
        n = int(fp.sum())
        if n == 0: continue
        ys, xs = np.nonzero(fp); y0, y1, x0, x1 = max(ys.min() - k - 1, 0), min(ys.max() + k + 2, H), max(xs.min() - k - 1, 0), min(xs.max() + k + 2, W)
        sub = fp[y0:y1, x0:x1]; near = (cv2.dilate(sub.astype(np.uint8), ker) > 0) & ~sub & solid[y0:y1, x0:x1]
        ring = near & (pm[y0:y1, x0:x1] < 0)
        if ring.sum() < 2: ring = near & (pm[y0:y1, x0:x1] != i)
        if ring.sum() < 2: continue
        gs = g[y0:y1, x0:x1]; d = D[y0:y1, x0:x1]
        out.append(dict(mark=int(i), px=n, contrast=float(gs[sub].mean() - gs[ring].mean()), effect=float(np.abs(d[sub] @ tl.GREY).mean()),
                        box=(y0, y1, x0, x1), fp=sub, ring=ring, d=d, span=int(max(np.ptp(ys), np.ptp(xs)) + 1)))
    return out
def presence(other_lit, lit0, inst, k):
    """How much of the mark's effect the OTHER picture shows at this footprint: the projection of (other - real with
    marks out), less its mean over the ring (broad tone is not detail), on the mark's effect; 1 = all of it, 0 = none.
    Best over shifts of up to k pixels."""
    H, W = other_lit.shape[:2]; y0, y1, x0, x1 = inst['box']; fp, ring, d = inst['fp'], inst['ring'], inst['d']
    d = d - d[ring].mean(0)                                  # the mark's effect against its own surroundings
    base = lit0[y0:y1, x0:x1].astype(float); den = float((d[fp] ** 2).sum())
    if den <= 0: return 0.0
    best = 0.0
    for dy in range(-k, k + 1):
        for dx in range(-k, k + 1):
            a0, a1, b0, b1 = y0 + dy, y1 + dy, x0 + dx, x1 + dx
            if a0 < 0 or b0 < 0 or a1 > H or b1 > W: continue
            o = other_lit[a0:a1, b0:b1]; ms = (o != L.BG).any(2); e = o.astype(float) - base
            ri = ring & ms
            if ri.sum() < max(1, .5 * ring.sum()) or (fp & ms).sum() < max(1, .5 * fp.sum()): continue
            e = e - e[ri].mean(0); e[~ms] = 0
            best = max(best, float((e[fp] * d[fp]).sum() / den))
    return best

def compare(a_ref, marks, pics_ref, pics_ref0, pics_other, npc, cams=('game', 'close', 'board')):
    """Follow every mark of the reference mesh into its pictures and look for it in the other's. For each instance the
    shift allowed is the largest of k, .., 1, 0 at which the reference WITH ITS MARKS PAINTED OUT does not itself show
    a quarter of the mark (a neighbouring mark sliding into the footprint is not this mark)."""
    allc, sc = cameras(npc); rec = [dict(inst={c: [] for c in cams}) for _ in marks]
    for key in pics_ref:
        cn, yaw, row, kind = key.split('|')
        if kind != 'face' or cn not in cams: continue
        lk = f'{cn}|{yaw}|{row}|lit'; lit, lit0, other = pics_ref[lk], pics_ref0[lk], pics_other[lk]; k = allc[cn]['k']
        for inst in instances(a_ref, marks, pics_ref[key], lit, lit0, k):
            ku = 0
            for kk in range(k, 0, -1):
                if presence(lit0, lit0, inst, kk) < 0.25: ku = kk; break
            t = presence(other, lit0, inst, ku)
            rec[inst['mark']]['inst'][cn].append(dict(view=f'{yaw}|{row}', px=inst['px'], span=inst['span'], contrast=inst['contrast'], effect=inst['effect'], k=ku, t=t))
    for r in rec:
        for cn in cams:
            vis = [i for i in r['inst'][cn] if i['px'] >= PX_MIN and i['effect'] >= C_MIN]
            r[cn] = dict(seen=len(r['inst'][cn]), visible=len(vis), max_px=max([i['px'] for i in r['inst'][cn]], default=0),
                         max_con=max([i['effect'] for i in vis], default=0.0),
                         kept=(float(np.mean([i['t'] >= HALF for i in vis])) if vis else None),
                         ratio=(float(np.median([min(i['t'], 1.0) for i in vis])) if vis else None))
    return rec
