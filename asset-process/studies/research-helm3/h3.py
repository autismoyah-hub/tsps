"""h3.py: the 17 pre-registered measures of PREREG.md -> data/h3.json.   Private: cache-derived.
usage: h3.py [KEY ...]      (default: the 14 helms of the test, plus the 5 ACC+ helms for reference)
Reuses research-helm2/h2.py (registry, outfits, drawing) read-only; every picture is written under this folder.
"""
import sys, pathlib, json, collections
sys.dont_write_bytecode = True
HERE = pathlib.Path(__file__).resolve().parent
H2DIR = HERE.parent / 'research-helm2'
sys.path.insert(0, str(H2DIR))
import numpy as np
from scipy import ndimage
from scipy.spatial import cKDTree
from PIL import Image, ImageDraw
import h2
from h2 import L
import fitlib

h2.WORK = HERE / 'work'                      # never draw into the helm2 folder
DATA = HERE / 'data'; DATA.mkdir(exist_ok=True)
UC = f'{h2.BB}/umbral-helm-c'
h2.ours('umbralC', 'RR', 'Umbral C', f'{UC}/umbral-helm-c-labelled.bin', 'head+hair+jaw', h2.folder_set(UC), f'{UC}/cuirass.bin')

ACC = ['torva', 'oathplate', 'justiciar', 'faceguard', 'serpentine', 'v5']
REJ = ['umbralA', 'umbralB', 'umbralC', 'v4a', 'v4b', 'v4c', 'v3a', 'v2a']
ACCP = ['inquisitor', 'masori', 'dharok', 'statius', 'armadyl']
ALL = ACC + REJ
VIEWS = {0: 'front', 256: 'quarter', 512: 'side'}


# ------------------------------------------------------------------------------------------------ pictures
def pictures(keys, tag='p'):
    es = []
    for k in keys: es += [h2.entry(k), h2.entry(k, helm=False)]
    out = {}
    for zn, zoom, cell in (('g', 300, (170, 170)), ('c', 130, (320, 340))):
        for yaw in VIEWS:
            a = h2.draw(f'{tag}-{zn}-{yaw}', es, yaw=yaw, zoom=zoom, cell=cell, aim='head')
            cs = h2.cells(a, len(es))
            for i, k in enumerate(keys): out[k, zn, yaw] = (cs[2 * i], cs[2 * i + 1])
    return out


def helm_cell(w, wo):
    bg = w[2, 2]
    M = h2.helm_mask(w, wo)
    lab, n = ndimage.label(M, np.ones((3, 3)))
    if n > 1:
        sz = ndimage.sum(M, lab, range(1, n + 1)); keep = [i + 1 for i, s in enumerate(sz) if s >= max(6, 0.02 * sz.max())]
        M = np.isin(lab, keep)
    return M, bg


def pairs(G, M):
    """4-neighbour pair differences with both pixels in M -> (d array, pixel-strong map builder)"""
    dh = np.abs(G[:, 1:] - G[:, :-1]); mh = M[:, 1:] & M[:, :-1]
    dv = np.abs(G[1:, :] - G[:-1, :]); mv = M[1:, :] & M[:-1, :]
    return dh, mh, dv, mv


def strong_px(G, M, T):
    dh, mh, dv, mv = pairs(G, M)
    S = np.zeros_like(M)
    h = (dh >= T) & mh; v = (dv >= T) & mv
    S[:, 1:] |= h; S[:, :-1] |= h; S[1:, :] |= v; S[:-1, :] |= v
    return S


def pix(key, P):
    r = {}
    hard, ramp, l90, surv = [], [], [], []
    for yaw, vn in VIEWS.items():
        dens = {}
        for zn in ('g', 'c'):
            w, wo = P[key, zn, yaw]; M, bg = helm_cell(w, wo); G = h2.grey(w)
            dh, mh, dv, mv = pairs(G, M); d = np.r_[dh[mh], dv[mv]]
            if zn == 'c':
                s12 = (d >= 12).sum(); hard.append(float((d >= 40).sum() / max(s12, 1)))
            else:
                s3 = (d >= 3).sum(); ramp.append(float(((d >= 3) & (d < 12)).sum() / max(s3, 1)))
                l90.append(float(np.percentile(d, 90)))
            dens[zn] = strong_px(G, M, 24).sum() / max(M.sum(), 1)
        surv.append(float(dens['g'] / max(dens['c'], 1e-6)))
    r['cr_hard_step_close'] = round(float(np.mean(hard)), 3)
    r['cr_ramp_game'] = round(float(np.mean(ramp)), 3)
    r['df_line_contrast_game'] = round(float(np.mean(l90)), 1)
    r['df_survive'] = round(float(np.mean(surv)), 3)
    # steel shading, close quarter
    w, wo = P[key, 'c', 256]; M, bg = helm_cell(w, wo); G = h2.grey(w)
    hh, ss, ll, ok = L.hsl_of(w[M])
    fam = np.where(ss <= 1, -1, hh)
    lit = ok & ~((hh == 4) & (ss == 3))
    main = collections.Counter(fam[lit].tolist()).most_common(1)[0][0]
    g = G[M][lit & (fam == main)]
    r['mo_steel_shading'] = round(float(np.percentile(g, 90) - np.percentile(g, 10)), 1)
    # dark cut, game front
    w, wo = P[key, 'g', 0]; M, bg = helm_cell(w, wo); G = h2.grey(w)
    p75 = np.percentile(G[M], 75); dk = M & (G < 16); lt = M & (G > p75)
    near = ndimage.binary_dilation(lt, np.ones((3, 3)))
    r['df_dark_cut'] = round(float((dk & near).sum() / dk.sum()), 3) if dk.sum() else 0.0
    return r


# ------------------------------------------------------------------------------------------------ mesh
def shell(mh):
    V = mh['V']
    keep = [i for i, f in enumerate(mh['F']) if len(set(f)) == 3 and not fitlib.wearer_face(mh['col'][i], mh['raw'][i])]
    key = {}; wid = {}
    for vi in range(len(V)):
        wid[vi] = key.setdefault(tuple(np.round(V[vi], 3)), len(key))
    P = np.zeros((len(key), 3))
    for vi, w in wid.items(): P[w] = V[vi]
    F = []
    for i in keep:
        a, b, c = (wid[v] for v in mh['F'][i])
        if len({a, b, c}) == 3: F.append((a, b, c))
    F = np.array(F)
    used = np.unique(F); remap = -np.ones(len(P), int); remap[used] = np.arange(len(used))
    return P[used], remap[F]


def topo(P, F):
    T = P[F]; n = np.cross(T[:, 1] - T[:, 0], T[:, 2] - T[:, 0]); area = np.linalg.norm(n, axis=1) / 2
    n = n / np.maximum(np.linalg.norm(n, axis=1)[:, None], 1e-12)
    ed = collections.defaultdict(list)
    for i, f in enumerate(F):
        for a, b in ((0, 1), (1, 2), (2, 0)):
            ed[tuple(sorted((int(f[a]), int(f[b]))))].append(i)
    fold = {}
    for e, fs in ed.items():
        if len(fs) == 2:
            fold[e] = float(np.degrees(np.arccos(np.clip(n[fs[0]] @ n[fs[1]], -1, 1))))
    ln = {e: float(np.linalg.norm(P[e[0]] - P[e[1]])) for e in ed}
    return T, n, area, ed, fold, ln


def plates(F, ed, fold, area):
    par = list(range(len(F)))
    def fd(x):
        while par[x] != x: par[x] = par[par[x]]; x = par[x]
        return x
    for e, fs in ed.items():
        if len(fs) == 2 and fold[e] < 20: par[fd(fs[0])] = fd(fs[1])
    g = collections.defaultdict(list)
    for i in range(len(F)): g[fd(i)].append(i)
    return list(g.values())


def wmedian(v, w):
    o = np.argsort(v); c = np.cumsum(np.asarray(w)[o]) / np.sum(w); return float(np.asarray(v)[o][np.searchsorted(c, 0.5)])


def chains(P, ed, fold, ln, thr=35.0, turn=35.0):
    hard = [e for e, a in fold.items() if a > thr]
    at = collections.defaultdict(list)
    for e in hard: at[e[0]].append(e); at[e[1]].append(e)
    link = collections.defaultdict(set)
    for v, es in at.items():
        cand = []
        for i in range(len(es)):
            for j in range(i + 1, len(es)):
                a, b = es[i], es[j]
                ua = P[v] - P[a[0] if a[1] == v else a[1]]; ub = P[b[0] if b[1] == v else b[1]] - P[v]
                t = np.degrees(np.arccos(np.clip(ua @ ub / max(np.linalg.norm(ua) * np.linalg.norm(ub), 1e-12), -1, 1)))
                cand.append((t, a, b))
        used = set()
        for t, a, b in sorted(cand, key=lambda x: x[0]):
            if t < turn and a not in used and b not in used:
                used |= {a, b}; link[a].add(b); link[b].add(a)
    seen = set(); out = []
    for e in hard:
        if e in seen: continue
        st = [e]; seen.add(e); tot = 0.0
        while st:
            x = st.pop(); tot += ln[x]
            for y in link[x]:
                if y not in seen: seen.add(y); st.append(y)
        out.append(tot)
    boundary = set()
    for e, fs in ed.items():
        if len(fs) == 1: boundary |= set(e)
    dangling = sum(1 for v, es in at.items() if len(es) == 1 and v not in boundary)
    total = sum(ln[e] for e in hard)
    return out, dangling, total, at


def raster(T, ax, box, S=4.0):
    (u0, u1), (y0, y1) = box
    W, Hh = int((u1 - u0) * S) + 1, int((y1 - y0) * S) + 1
    im = Image.new('1', (W, Hh), 0); d = ImageDraw.Draw(im)
    ui = 0 if ax == 'front' else 2
    for t in T: d.polygon([((p[ui] - u0) * S, (y1 - p[1]) * S) for p in t], fill=1, outline=1)
    return np.asarray(im).astype(bool)


def sil_dev(T, ax):
    S = 4.0; box = ((-45, 45), (150, 235))
    A = raster(T, ax, box, S); Hd = raster(fitlib.head_tris(), ax, box, S)
    A = A & ndimage.binary_dilation(Hd, iterations=int(3.5 * S))
    lab, n = ndimage.label(A)
    if n > 1:
        sz = ndimage.sum(A, lab, range(1, n + 1)); A = lab == (1 + int(np.argmax(sz)))
    A = ndimage.binary_fill_holes(A)
    ys, xs = np.nonzero(A); cy, cx = ys.mean(), xs.mean()
    th = np.linspace(0, 2 * np.pi, 360, endpoint=False)
    rr = np.arange(0, 400) * 0.5
    r = np.zeros(360)
    for k, t in enumerate(th):
        yy = np.round(cy - rr * np.sin(t)).astype(int); xx = np.round(cx + rr * np.cos(t)).astype(int)
        ok = (yy >= 0) & (yy < A.shape[0]) & (xx >= 0) & (xx < A.shape[1])
        hit = np.zeros(len(rr), bool); hit[ok] = A[yy[ok], xx[ok]]
        r[k] = rr[np.nonzero(hit)[0].max()] if hit.any() else 0
    Fq = np.fft.rfft(r); Fq[7:] = 0; lp = np.fft.irfft(Fq, 360)
    return float(np.sqrt(np.mean((r - lp) ** 2)) / r.mean()), A, (cy, cx), r, lp


def mesh_measures(mh):
    P, F = shell(mh); T, n, area, ed, fold, ln = topo(P, F)
    r = {}
    tl = sum(ln[e] for e in fold)
    r['cr_soft_fold'] = round(sum(ln[e] for e, a in fold.items() if 15 <= a <= 40) / tl, 3)
    # relief
    nb = collections.defaultdict(set)
    for e in ed: nb[e[0]].add(e[1]); nb[e[1]].add(e[0])
    Q = P.copy()
    for _ in range(20):
        M = np.array([Q[list(nb[i])].mean(0) for i in range(len(Q))]); Q = Q + 0.5 * (M - Q)
    va = np.zeros(len(P))
    for i, f in enumerate(F):
        for v in f: va[v] += area[i] / 3
    r['mo_relief'] = round(float((np.linalg.norm(P - Q, axis=1) * va).sum() / va.sum() / np.sqrt(area.sum())), 4)
    # plates
    cvs, ws = [], []
    for pl in plates(F, ed, fold, area):
        if len(pl) < 4: continue
        es = {tuple(sorted((int(F[i][a]), int(F[i][b])))) for i in pl for a, b in ((0, 1), (1, 2), (2, 0))}
        L_ = np.array([ln[e] for e in es]); cvs.append(L_.std() / L_.mean()); ws.append(area[pl].sum())
    r['un_edge_cv_plate'] = round(wmedian(cvs, ws), 3) if cvs else None
    r['un_face_logsd'] = round(float(np.std(np.log(np.maximum(area, 1e-6)))), 3)
    # asymmetry
    rng = np.random.default_rng(0)
    pick = rng.choice(len(T), 600, p=area / area.sum())
    u = rng.random((600, 2)); m = u.sum(1) > 1; u[m] = 1 - u[m]
    X = T[pick, 0] + u[:, :1] * (T[pick, 1] - T[pick, 0]) + u[:, 1:] * (T[pick, 2] - T[pick, 0])
    Xm = X * np.array([-1, 1, 1])
    dd, _ = cKDTree(X).query(Xm)
    r['un_asym'] = round(float(dd.mean() / (P[:, 0].max() - P[:, 0].min())), 4)
    # silhouettes
    r['ms_sil_front'] = round(sil_dev(T, 'front')[0], 4)
    r['ms_sil_side'] = round(sil_dev(T, 'side')[0], 4)
    # dents
    vn = np.zeros((len(P), 3))
    for i, f in enumerate(F):
        for v in f: vn[v] += n[i] * area[i]
    vn /= np.maximum(np.linalg.norm(vn, axis=1)[:, None], 1e-12)
    smooth = np.ones(len(P), bool)
    for e, fs in ed.items():
        a = fold.get(e)
        if a is None or a >= 30: smooth[e[0]] = smooth[e[1]] = False
    sg = np.zeros(len(P), int)
    for i in np.nonzero(smooth)[0]:
        s = (P[i] - P[list(nb[i])].mean(0)) @ vn[i]
        sg[i] = 0 if abs(s) < 0.05 else (1 if s > 0 else -1)
    se = [e for e in ed if smooth[e[0]] and smooth[e[1]]]
    r['ms_dents'] = round(sum(1 for e in se if sg[e[0]] * sg[e[1]] == -1) / len(se), 3) if se else 0.0
    r['diag_dents_edges'] = len(se); r['diag_smooth_vertices'] = int(smooth.sum()); r['diag_verts'] = len(P)
    # chains
    cl, dang, tot, _ = chains(P, ed, fold, ln)
    r['in_long_lines'] = round(sum(c for c in cl if c >= 12) / max(tot, 1e-9), 3)
    r['in_dangling'] = round(100.0 * dang / max(tot, 1e-9), 2)
    # mirror of hard folds
    hard = [e for e, a in fold.items() if a > 45]
    mid = np.array([(P[e[0]] + P[e[1]]) / 2 for e in hard]); hl = np.array([ln[e] for e in hard])
    tr = cKDTree(mid); good = 0.0
    for k, p in enumerate(mid * np.array([-1, 1, 1])):
        js = tr.query_ball_point(p, 1.0)
        if any(j != k for j in js): good += hl[k]
    r['in_mirror'] = round(float(good / hl.sum()), 3)
    return r


MEASURES = ['cr_hard_step_close', 'cr_ramp_game', 'cr_soft_fold', 'mo_relief', 'mo_steel_shading',
            'df_line_contrast_game', 'df_survive', 'df_dark_cut', 'un_edge_cv_plate', 'un_face_logsd', 'un_asym',
            'ms_sil_front', 'ms_sil_side', 'ms_dents', 'in_long_lines', 'in_dangling', 'in_mirror']
DIR = dict(cr_hard_step_close=+1, cr_ramp_game=-1, cr_soft_fold=-1, mo_relief=+1, mo_steel_shading=+1,
           df_line_contrast_game=+1, df_survive=+1, df_dark_cut=+1, un_edge_cv_plate=-1, un_face_logsd=-1, un_asym=-1,
           ms_sil_front=-1, ms_sil_side=-1, ms_dents=-1, in_long_lines=+1, in_dangling=-1, in_mirror=+1)

if __name__ == '__main__':
    keys = sys.argv[1:] or ALL + ACCP
    P = pictures(keys)
    f = DATA / 'h3.json'
    out = json.load(open(f)) if f.exists() else {}
    for k in keys:
        r = dict(group=('ACC' if k in ACC else 'REJ' if k in REJ else 'ACC+'), name=h2.H[k]['name'])
        r.update(pix(k, P)); r.update(mesh_measures(h2.mesh(k)))
        out[k] = r; print(k, {m: r[m] for m in MEASURES})
    json.dump(out, open(f, 'w'), indent=1)
