"""research-paint library: how PAINT (per-face colour) is ORGANISED on a creature, beyond counts.

    sk = skin('npc:7541')  or  skin('def:PATH.json')      the drawn, opaque skin of the REST mesh as the client assembles it
    a = analyse(sk)        -> dict(creature measures 'm', per-part table 'parts', materials 'mats', islands, ...)
    paint_view(sk, yaw, pitch, px_per_unit)               unlit paint picture (own z-buffer; colours by the game's table)

Coordinates are the cache's: x left-right, y DOWN, z along the body (front = -z). Colours are 16-bit HSL.
A face belongs to the SKIN if it is drawn (alpha not 255, render type under 2), opaque (alpha 0) and has area.
Translucent faces (veils, glows, pools) are left out of every paint measure; their share is reported.

Terms used everywhere in this folder:
  material  a group of colours linked by neighbouring hue and saturation (hue +-1, saturation +-1; greys join greys):
            one painted substance (hide, bone, iron, glow). Its colours sorted by lightness are its RAMP.
  part      a zone of the body. Default: 8 zones found from the face centres alone (so labelled and bone-weighted
            creatures are cut alike); a paint plan may name parts by vertex labels instead.
  patch     a connected region of one colour.
  mark      a patch up to 1.2% of the skin that contrasts with what surrounds it (8 lightness or another material).
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, pathlib, json, math, collections
import numpy as np
HERE = pathlib.Path(__file__).resolve().parent
B = HERE.parent
sys.path.insert(0, str(B / 'research-creatures')); sys.path.insert(1, str(B / 'research-npc')); sys.path.insert(2, _AP.fix(r'<ROOT>\author-pack\tools'))
import cmesh, gamecol
cmesh.DATA = HERE / 'data'                      # every cached file of this study stays in this folder
TABLE = gamecol.table()
NPARTS = 8
MARK_MAX = 0.012
SIG = 0.003                                     # a colour counts as a tone of its ramp from 0.3% of the skin

def hsl(c): return (int(c) >> 10) & 63, (int(c) >> 7) & 7, int(c) & 127
def pack(h, s, l): return (int(h) << 10) | (int(s) << 7) | int(l)
def rgb(c): return tuple(int(x) for x in TABLE[int(c)])

# ------------------------------------------------------------------------------------------------ loading
def skin(entry, mesh=None):
    m = mesh if mesh is not None else cmesh.load(entry)
    area, unit, cen = cmesh.face_geometry(m)
    sign = cmesh.outward_sign(m, area, unit, cen)
    drawn = (m['A'] != 255) & (m['T'] < 2) & (area > 0)
    keep = drawn & (m['A'] == 0)
    idx = np.nonzero(keep)[0]
    sk = dict(entry=entry, V=m['V'], L=m['L'], F=m['F'][keep], C=m['C'][keep], area=area[keep], unit=unit[keep] * sign, cen=cen[keep], idx=idx,
              translucent_share=float(area[drawn & (m['A'] != 0)].sum() / max(area[drawn].sum(), 1e-9)), faces_drawn=int(drawn.sum()), mesh=m)
    sk['tot'] = float(sk['area'].sum())
    _topology(sk)
    return sk

def _topology(sk):
    F, C = sk['F'], sk['C']; nf = len(F)
    edges = collections.defaultdict(list)
    for i, f in enumerate(F):
        for a, b in ((f[0], f[1]), (f[1], f[2]), (f[2], f[0])): edges[(min(a, b), max(a, b))].append(i)
    e2 = [(k, v) for k, v in edges.items() if len(v) == 2]
    sk['e_a'] = np.array([v[0] for _, v in e2], int); sk['e_b'] = np.array([v[1] for _, v in e2], int)
    ev = np.array([k for k, _ in e2], int).reshape(-1, 2); sk['e_v'] = ev
    V = sk['V']
    sk['e_len'] = np.linalg.norm(V[ev[:, 0]] - V[ev[:, 1]], axis=1) if len(ev) else np.zeros(0)
    ua, ub = sk['unit'][sk['e_a']], sk['unit'][sk['e_b']]
    sk['e_fold'] = np.degrees(np.arccos(np.clip((ua * ub).sum(1), -1, 1)))
    # convex if the other face's centre lies behind this face's plane
    sk['e_convex'] = ((sk['cen'][sk['e_b']] - sk['cen'][sk['e_a']]) * ua).sum(1) < 0
    sk['open_edges'] = [k for k, v in edges.items() if len(v) == 1]
    parent = list(range(nf))
    def find(i):
        while parent[i] != i: parent[i] = parent[parent[i]]; i = parent[i]
        return i
    for a, b in zip(sk['e_a'], sk['e_b']):
        if C[a] == C[b]: parent[find(b)] = find(a)
    roots = np.array([find(i) for i in range(nf)], int)
    u, inv = np.unique(roots, return_inverse=True)
    sk['patch'] = inv; sk['npatch'] = len(u)

# ------------------------------------------------------------------------------------------------ materials (ramps)
def near(c, seed):
    """Distance of colour c from a material's seed colour, or None if it cannot be that material: hue within 2 steps
    and saturation within 2 of the seed (4 for very dark or very pale tones, which show little hue); greys
    (saturation 0 or 1) are one substance; black (lightness up to 4) is its own."""
    h, s, l = hsl(c); hs, ss, ls_ = hsl(seed)
    if ls_ <= 4: return 0.0 if l <= 4 else None
    if l <= 4: return None
    dh = min(abs(h - hs), 64 - abs(h - hs)); ds = abs(s - ss)
    if max(s, ss) <= 1 and (min(s, ss) == 0 or dh <= 4): return 0.5 * ds
    if min(s, ss) == 0: return None
    loose = l <= 12 or l >= 112
    if dh <= (4 if loose else 2) and ds <= (4 if loose else 2): return dh + 0.5 * ds
    return None

def materials(sk):
    """Colours near one seed colour in hue and saturation form one material (seeds = the largest colours first).
    Returns the materials, largest first, and an array giving each face's material index."""
    C, area, tot = sk['C'], sk['area'], sk['tot']
    by = collections.Counter()
    for c, a in zip(C, area): by[int(c)] += a
    cols = [c for c, _ in by.most_common()]
    seeds = []; groups = collections.defaultdict(list)
    for c in cols:                                   # largest colours first: each founds a material or joins the nearest
        best = None
        for sd in seeds:
            d = near(c, sd)
            if d is not None and (best is None or d < best[0]): best = (d, sd)
        if best is None: seeds.append(c); groups[c].append(c)
        else: groups[best[1]].append(c)
    mats = []
    for g in groups.values():
        share = sum(by[c] for c in g) / tot
        sig = sorted([c for c in g if by[c] / tot >= SIG], key=lambda c: (hsl(c)[2], c))
        ls = sorted({hsl(c)[2] for c in sig})
        steps = np.diff(ls) if len(ls) > 1 else np.zeros(0)
        w = np.array([by[c] for c in g]); H = np.array([hsl(c)[0] for c in g], float); S = np.array([hsl(c)[1] for c in g], float); Lg = np.array([hsl(c)[2] for c in g], float)
        h0 = H[np.argmax(w)]; Hc = ((H - h0 + 32) % 64) - 32                                   # hue offset from the main colour
        mats.append(dict(colours=sorted(g, key=lambda c: (hsl(c)[2], c)), share=share, tones=len(ls), lights=ls,
                         step_med=float(np.median(steps)) if len(steps) else 0.0,
                         step_cv=float(steps.std() / steps.mean()) if len(steps) > 1 and steps.mean() > 0 else 0.0,
                         span=float(ls[-1] - ls[0]) if ls else 0.0,
                         hue=int(h0), sat=float(np.average(S, weights=w)), light=float(np.average(Lg, weights=w)),
                         hue_drift=cmesh.wcorr(Lg, Hc, w) or 0.0, sat_drift=cmesh.wcorr(Lg, S, w) or 0.0,
                         hues=len({hsl(c)[0] for c in sig}), sats=len({hsl(c)[1] for c in sig}),
                         shares={int(c): by[c] / tot for c in g}))
    mats.sort(key=lambda d: -d['share'])
    cm = {c: i for i, d in enumerate(mats) for c in d['colours']}
    return mats, np.array([cm[int(c)] for c in C], int)

def mat_name(d):
    h, s, l = d['hue'], d['sat'], d['light']
    if l <= 6: return 'black'
    if s < 0.6: tone = 'grey'
    else:
        names = [(1, 'red'), (4, 'orange'), (7, 'brown-gold'), (10, 'yellow'), (16, 'olive-green'), (25, 'green'), (33, 'teal'), (41, 'blue'), (47, 'violet'), (55, 'purple'), (61, 'crimson'), (64, 'red')]
        tone = next(n for k, n in names if h <= k)
    depth = 'dark ' if l < 28 else 'pale ' if l > 78 else ''
    return depth + tone

# ------------------------------------------------------------------------------------------------ parts
def auto_parts(sk, k=NPARTS):
    """k zones from the face centres (area-weighted k-means, farthest-point start: the same cut every run)."""
    X, w = sk['cen'], sk['area']
    if len(X) <= k: return np.arange(len(X)) % k, X[:k].copy()
    mu = np.average(X, axis=0, weights=w)
    seeds = [int(np.argmax(((X - mu) ** 2).sum(1)))]
    d = ((X - X[seeds[0]]) ** 2).sum(1)
    for _ in range(k - 1):
        seeds.append(int(np.argmax(d))); d = np.minimum(d, ((X - X[seeds[-1]]) ** 2).sum(1))
    cen = X[seeds].copy()
    for _ in range(40):
        lab = np.argmin(((X[:, None, :] - cen[None]) ** 2).sum(2), axis=1)
        new = np.array([np.average(X[lab == j], axis=0, weights=w[lab == j]) if (lab == j).any() else cen[j] for j in range(k)])
        if np.allclose(new, cen, atol=0.01): break
        cen = new
    return lab, cen

def zone_names(sk, lab, cen):
    V = sk['V'][np.unique(sk['F'])]; lo, hi = V.min(0), V.max(0); ext = np.maximum(hi - lo, 1)
    names = []
    for j, c in enumerate(cen):
        r = (c - lo) / ext
        v = 'top' if r[1] < 0.34 else 'low' if r[1] > 0.66 else 'mid'
        x = c[0] - (lo[0] + hi[0]) / 2
        s = 'left' if x < -0.12 * ext[0] else 'right' if x > 0.12 * ext[0] else 'centre'       # the creature's own -x / +x
        f = 'front' if r[2] < 0.34 else 'rear' if r[2] > 0.66 else ''
        n = '-'.join(t for t in (v, s, f) if t)
        while n in names: n += "'"
        names.append(n)
    return names

def label_parts(sk, spec):
    """spec: {name: [labels]} ; a face goes to the part holding the majority of its three vertex labels; the rest -> 'other'."""
    names = list(spec) + ['other']; lut = {}
    for i, n in enumerate(spec):
        for l in spec[n]: lut[int(l)] = i
    fl = sk['L'][sk['F']]
    out = np.full(len(fl), len(spec), int)
    for i, tri in enumerate(fl):
        c = collections.Counter(lut.get(int(l), len(spec)) for l in tri); out[i] = c.most_common(1)[0][0]
    return out, names

# ------------------------------------------------------------------------------------------------ marks and motifs
MOTIFS = ('light line', 'dark line', 'accent line', 'light spot', 'dark spot', 'accent spot')
def marks(sk, mat_of):
    """Small contrasting patches with shape. Returns list of dicts (faces, area, length, width, kind, part filled later)."""
    C, area, patch, tot = sk['C'], sk['area'], sk['patch'], sk['tot']
    np_ = sk['npatch']
    p_area = np.bincount(patch, weights=area, minlength=np_)
    p_col = np.zeros(np_, int); p_col[patch] = C
    p_mat = np.zeros(np_, int); p_mat[patch] = mat_of
    nb = collections.defaultdict(lambda: collections.Counter())
    pa, pb = patch[sk['e_a']], patch[sk['e_b']]
    for a, b, l in zip(pa, pb, sk['e_len']):
        if a != b: nb[a][b] += l; nb[b][a] += l
    faces_of = collections.defaultdict(list)
    for i, p in enumerate(patch): faces_of[p].append(i)
    out = []
    V, F = sk['V'], sk['F']
    for p in range(np_):
        if p_area[p] > MARK_MAX * tot or not nb[p]: continue
        l = hsl(p_col[p])[2]; tl_ = sum(nb[p].values())
        nl = sum(hsl(p_col[q])[2] * w for q, w in nb[p].items()) / tl_
        other = sum(w for q, w in nb[p].items() if p_mat[q] != p_mat[p]) / tl_
        accent = other >= 0.7
        if not accent and abs(l - nl) < 8: continue
        fs = faces_of[p]
        pts = V[F[fs]].reshape(-1, 3)
        if len(fs) == 1:
            e = np.sort([np.linalg.norm(pts[0] - pts[1]), np.linalg.norm(pts[1] - pts[2]), np.linalg.norm(pts[2] - pts[0])])
            length = float(e[2]); width = float(2 * p_area[p] / max(e[2], 1e-9))
        else:
            ev = np.linalg.eigvalsh(np.cov((pts - pts.mean(0)).T) + 1e-9 * np.eye(3))
            length = float(3.46 * math.sqrt(max(ev[2], 0))); width = float(max(p_area[p] / max(length, 1e-9), 3.46 * math.sqrt(max(ev[1], 0)) * 0.5))
        line = length / max(width, 1e-9) >= 3.0
        kind = ('accent ' if accent else 'light ' if l > nl else 'dark ') + ('line' if line else 'spot')
        out.append(dict(patch=p, faces=fs, area=float(p_area[p]), length=length, width=width, kind=kind, contrast=float(l - nl), colour=int(p_col[p]),
                        cen=np.average(sk['cen'][fs], axis=0, weights=area[fs])))
    return out

# ------------------------------------------------------------------------------------------------ the analysis
def _hist_inter(p, q):
    keys = set(p) | set(q)
    return sum(min(p.get(k, 0.0), q.get(k, 0.0)) for k in keys)

def analyse(sk, part=None, names=None, k=NPARTS):
    C, area, tot, cen = sk['C'], sk['area'], sk['tot'], sk['cen']
    mats, mat_of = materials(sk)
    if part is None:
        part, pc = auto_parts(sk, k); names = zone_names(sk, part, pc)
    npart = len(names)
    Ls = np.array([hsl(c)[2] for c in C], float)
    ea, eb, el, fold, convex = sk['e_a'], sk['e_b'], sk['e_len'], sk['e_fold'], sk['e_convex']
    change = C[ea] != C[eb]
    same_mat = mat_of[ea] == mat_of[eb]
    mk = marks(sk, mat_of)
    for d in mk: d['part'] = int(collections.Counter(part[d['faces']].tolist()).most_common(1)[0][0])
    p_area = np.bincount(sk['patch'], weights=area, minlength=sk['npatch'])
    p_n = np.bincount(sk['patch'], minlength=sk['npatch'])
    # ---- per part
    rows = []
    for j in range(npart):
        s = part == j; a = float(area[s].sum())
        if a <= 0: rows.append(dict(name=names[j], share=0.0)); continue
        r = dict(name=names[j], share=a / tot, faces=int(s.sum()))
        col = collections.Counter()
        for c, w in zip(C[s], area[s]): col[int(c)] += w / a
        r['colours'] = dict(col)
        r['n_colours'] = sum(1 for v in col.values() if v >= 0.01)
        mshare = np.bincount(mat_of[s], weights=area[s], minlength=len(mats)) / a
        r['mat_share'] = mshare.tolist(); mm = int(np.argmax(mshare)); r['main_mat'] = mm
        sm = s & (mat_of == mm)
        r['tones_main'] = len({hsl(c)[2] for c, v in col.items() if v >= 0.01 and c in mats[mm]['shares']})
        r['light_mean'] = float(np.average(Ls[s], weights=area[s])); r['light_sd'] = float(math.sqrt(np.average((Ls[s] - r['light_mean']) ** 2, weights=area[s])))
        r['light_sd_main'] = float(math.sqrt(np.average((Ls[sm] - np.average(Ls[sm], weights=area[sm])) ** 2, weights=area[sm])))
        # edges inside the part
        es = s[ea] & s[eb]; L_in = el[es].sum()
        r['boundary_density'] = float(el[es & change].sum() / max(a, 1e-9) * math.sqrt(tot))       # paint boundary length per area, scale-free
        r['boundary_share'] = float(el[es & change].sum() / max(L_in, 1e-9))
        dl = np.abs(Ls[ea] - Ls[eb])[es & change & same_mat]
        r['step_med'] = float(np.median(dl)) if len(dl) >= 5 else None
        r['step_p90'] = float(np.quantile(dl, .9)) if len(dl) >= 5 else None
        r['cross_mat_boundary'] = float(el[es & change & ~same_mat].sum() / max(el[es & change].sum(), 1e-9))
        r['boundary_on_fold'] = float(el[es & change & (fold >= 20)].sum() / max(el[es & change].sum(), 1e-9))
        # patches of the part (a patch belongs to the part holding most of it; approximated by face membership)
        ps = np.unique(sk['patch'][s]); pa = p_area[ps] / tot * 1e4
        r['patch_med_e4'] = float(np.median(pa)); r['patch_logsd'] = float(np.log10(np.maximum(pa, 1e-6)).std())
        r['patch_area_med_e4'] = cmesh.wquant(p_area[sk['patch'][s]] / tot * 1e4, area[s], .5)          # the patch a random point of the part lies in
        r['speckle'] = float(area[s][p_n[sk['patch'][s]] <= 1].sum() / a)                                 # surface in single-face patches
        # gradient along the part's long axis, main material
        if sm.sum() >= 8:
            X = cen[sm]; w = area[sm]; mu = np.average(X, axis=0, weights=w)
            cov = np.cov((X - mu).T, aweights=w); evals, evecs = np.linalg.eigh(cov); ax = evecs[:, 2]
            t = (X - mu) @ ax
            r['gradient'] = abs(cmesh.wcorr(Ls[sm], t, w) or 0.0)
            eb_ = es & change
            if eb_.sum() >= 5:
                dv = sk['V'][sk['e_v'][eb_, 0]] - sk['V'][sk['e_v'][eb_, 1]]; dv /= np.maximum(np.linalg.norm(dv, axis=1), 1e-9)[:, None]
                r['boundary_along_axis'] = float(np.average(np.abs(dv @ ax), weights=el[eb_]))   # 0.5 = no direction; low = bands across; high = stripes along
        # edge treatment: sharp convex folds (plate edges, crests) and sharp concave folds (recesses)
        sharp = es & (fold >= 30)
        medL = {}
        for q in np.unique(mat_of[s]):
            t = s & (mat_of == q); medL[q] = cmesh.wquant(Ls[t], area[t], .5)
        if sharp.sum():
            hi = np.maximum(Ls[ea], Ls[eb]); lo = np.minimum(Ls[ea], Ls[eb])
            ref = cmesh.wquant(Ls[s], area[s], .5)            # the part's median tone, all materials
            cv, cc = sharp & convex, sharp & ~convex
            r['sharp_len'] = float(el[sharp].sum()); r['convex_len'] = float(el[cv].sum()); r['concave_len'] = float(el[cc].sum())
            r['edge_light'] = float(el[cv & (hi - ref >= 4)].sum() / max(el[cv].sum(), 1e-9)) if cv.sum() >= 5 else None
            r['recess_dark'] = float(el[cc & (lo - ref <= -4)].sum() / max(el[cc].sum(), 1e-9)) if cc.sum() >= 5 else None
            r['fold_paint'] = float(el[sharp & change].sum() / max(el[sharp].sum(), 1e-9))
        mine = [d for d in mk if d['part'] == j]
        r['marks'] = len(mine); r['mark_density'] = len(mine) / (a / tot) / max(len(mk), 1)        # 1 = the creature's average density
        r['motifs'] = {k_: sum(1 for d in mine if d['kind'] == k_) for k_ in MOTIFS}
        rows.append(r)
    big = [r for r in rows if r.get('share', 0) >= 0.03]
    w = np.array([r['share'] for r in big])
    def spread(key, log=False, rel=True):
        x = np.array([r.get(key) if r.get(key) is not None else np.nan for r in big], float); ok = np.isfinite(x)
        if ok.sum() < 3: return None
        x = x[ok]
        if log: return float(np.log10(np.maximum(x, 1e-6)).std())
        return float(x.std() / max(abs(x.mean()), 1e-9)) if rel else float(x.std())
    def wmean(key):
        x = np.array([r.get(key) if r.get(key) is not None else np.nan for r in big], float); ok = np.isfinite(x)
        return float(np.average(x[ok], weights=w[ok])) if ok.any() else None
    m = {}
    m['skin_faces'] = int(len(C)); m['translucent_share'] = sk['translucent_share']
    # ---- palette as structure
    main = [d for d in mats if d['share'] >= 0.03]
    m['materials'] = len(main); m['materials_8'] = sum(1 for d in mats if d['share'] >= 0.08)
    m['mat1_share'] = mats[0]['share']; m['mat2_share'] = mats[1]['share'] if len(mats) > 1 else 0.0; m['mat3_share'] = mats[2]['share'] if len(mats) > 2 else 0.0
    m['top2_share'] = m['mat1_share'] + m['mat2_share']
    m['minor_share'] = float(sum(d['share'] for d in mats if d['share'] < 0.03))
    m['minor_materials'] = sum(1 for d in mats if SIG <= d['share'] < 0.03)
    m['ramp1_tones'] = mats[0]['tones']; m['ramp1_step'] = mats[0]['step_med']; m['ramp1_step_cv'] = mats[0]['step_cv']; m['ramp1_span'] = mats[0]['span']
    m['ramp1_hues'] = mats[0]['hues']; m['ramp1_sats'] = mats[0]['sats']; m['ramp1_hue_drift'] = mats[0]['hue_drift']; m['ramp1_sat_drift'] = mats[0]['sat_drift']
    mw = np.array([d['share'] for d in main])
    m['ramp_tones_mean'] = float(np.average([d['tones'] for d in main], weights=mw))
    m['ramp_step_mean'] = float(np.average([d['step_med'] for d in main], weights=mw))
    m['ramp_step_cv_mean'] = float(np.average([d['step_cv'] for d in main], weights=mw))
    m['ramp_span_mean'] = float(np.average([d['span'] for d in main], weights=mw))
    m['ramp_hues_mean'] = float(np.average([d['hues'] for d in main], weights=mw))
    m['ramp_sat_drift_mean'] = float(np.average([d['sat_drift'] for d in main], weights=mw))
    m['colours_sig'] = sum(1 for d in mats for c, v in d['shares'].items() if v >= SIG)
    m['colours_per_material'] = m['colours_sig'] / max(m['materials'] + m['minor_materials'], 1)
    # ---- reuse of the palette across parts (CONSISTENCY)
    ncol_parts = collections.Counter(); col_share = collections.Counter()
    for r in big:
        for c, v in r['colours'].items():
            if v >= 0.01: ncol_parts[c] += 1
    for d in mats:
        for c, v in d['shares'].items(): col_share[c] = v
    half = max(2, math.ceil(len(big) / 2))
    m['shared_colour_share'] = float(sum(r['share'] * sum(v for c, v in r['colours'].items() if ncol_parts[c] >= half) for r in big) / w.sum())
    m['orphan_colour_share'] = float(sum(r['share'] * sum(v for c, v in r['colours'].items() if v >= 0.01 and ncol_parts[c] == 1) for r in big) / w.sum())
    m['orphan_colours'] = sum(1 for c, n in ncol_parts.items() if n == 1)
    m['orphan_colours_share_of_palette'] = m['orphan_colours'] / max(len(ncol_parts), 1)
    m['parts_per_colour'] = float(np.mean([n / len(big) for n in ncol_parts.values()])) if ncol_parts else 0.0
    sims, ws = [], []
    for i in range(len(big)):
        for j in range(i + 1, len(big)):
            sims.append(_hist_inter(big[i]['colours'], big[j]['colours'])); ws.append(big[i]['share'] * big[j]['share'])
    m['palette_overlap'] = float(np.average(sims, weights=ws)) if sims else None      # 1 = every part painted from the same colours in the same shares
    m['palette_overlap_min'] = float(min(max(_hist_inter(big[i]['colours'], big[j]['colours']) for j in range(len(big)) if j != i) for i in range(len(big)))) if len(big) > 1 else None
    msims = []
    for i in range(len(big)):
        for j in range(i + 1, len(big)):
            msims.append(float(np.minimum(big[i]['mat_share'], big[j]['mat_share']).sum()))
    m['material_overlap'] = float(np.average(msims, weights=ws)) if msims else None
    m['mat1_parts'] = sum(1 for r in big if r['mat_share'][0] >= 0.10) / len(big)
    m['mat2_parts'] = sum(1 for r in big if len(r['mat_share']) > 1 and r['mat_share'][1] >= 0.03) / len(big)
    m['mat3_parts'] = sum(1 for r in big if len(r['mat_share']) > 2 and r['mat_share'][2] >= 0.03) / len(big)
    m['colours_per_part'] = wmean('n_colours'); m['colours_per_part_cv'] = spread('n_colours')
    m['tones_main_per_part'] = wmean('tones_main'); m['tones_main_per_part_cv'] = spread('tones_main')
    # ---- tone inside a part
    m['step_med'] = float(np.median(np.abs(Ls[ea] - Ls[eb])[change & same_mat])) if (change & same_mat).sum() else 0.0
    m['step_part_sd'] = spread('step_med', rel=False); m['step_part_cv'] = spread('step_med')
    m['light_sd_part'] = wmean('light_sd_main'); m['light_sd_part_cv'] = spread('light_sd_main')
    m['light_mean_part_sd'] = spread('light_mean', rel=False)
    m['gradient'] = wmean('gradient')
    m['boundary_along_axis'] = wmean('boundary_along_axis')
    m['boundary_on_fold'] = float(el[change & (fold >= 20)].sum() / max(el[change].sum(), 1e-9))
    m['cross_mat_boundary'] = float(el[change & ~same_mat].sum() / max(el[change].sum(), 1e-9))
    pa = p_area / tot * 1e4
    m['patch_med_e4'] = float(np.median(pa)); m['patch_logsd'] = float(np.log10(np.maximum(pa, 1e-6)).std())
    m['patch_area_med_e4'] = cmesh.wquant(p_area[sk['patch']] / tot * 1e4, area, .5)
    m['patch_part_logsd'] = spread('patch_area_med_e4', log=True)
    m['patch_count_part_logsd'] = spread('patch_med_e4', log=True)
    m['speckle'] = float(area[p_n[sk['patch']] <= 1].sum() / tot); m['speckle_part_sd'] = spread('speckle', rel=False)
    m['boundary_share'] = float(el[change].sum() / max(el.sum(), 1e-9))
    m['boundary_density_cv'] = spread('boundary_density'); m['boundary_share_part_sd'] = spread('boundary_share', rel=False)
    # ---- edge treatment
    sharp = fold >= 30
    hi = np.maximum(Ls[ea], Ls[eb]); lo = np.minimum(Ls[ea], Ls[eb])
    m['sharp_edge_share'] = float(el[sharp].sum() / max(el.sum(), 1e-9))
    m['edge_light'] = wmean('edge_light'); m['edge_light_part_sd'] = spread('edge_light', rel=False)
    m['recess_dark'] = wmean('recess_dark'); m['recess_dark_part_sd'] = spread('recess_dark', rel=False)
    m['fold_paint'] = wmean('fold_paint'); m['fold_paint_part_sd'] = spread('fold_paint', rel=False)
    # ---- does tone follow FORM: paint changes at folds (painted planes) against paint changes inside a flat surface
    m.update(form_numbers(el, fold, change, same_mat, np.abs(Ls[ea] - Ls[eb])))
    for r in big:
        s = part == names.index(r['name']); es = s[ea] & s[eb]
        if es.sum() >= 30: r.update(form_numbers(el[es], fold[es], change[es], same_mat[es], np.abs(Ls[ea] - Ls[eb])[es]))
    m['planes_part_sd'] = spread('planes_toned', rel=False); m['planes_part_min'] = min((r['planes_toned'] for r in big if r.get('planes_toned') is not None), default=None)
    m['flat_part_max'] = max((r['flat_broken'] for r in big if r.get('flat_broken') is not None), default=None)
    # ---- marks and motifs
    m['marks'] = len(mk); m['mark_area_share'] = float(sum(d['area'] for d in mk) / tot)
    kinds = collections.Counter(d['kind'] for d in mk)
    for k_ in MOTIFS: m['share ' + k_] = kinds[k_] / max(len(mk), 1)
    used = [k_ for k_ in MOTIFS if kinds[k_] >= max(3, 0.05 * len(mk))]
    m['motifs_used'] = len(used)
    m['line_share'] = sum(kinds[k_] for k_ in MOTIFS if 'line' in k_) / max(len(mk), 1)
    m['accent_mark_share'] = sum(kinds[k_] for k_ in MOTIFS if 'accent' in k_) / max(len(mk), 1)
    rec = [sum(1 for r in big if r['motifs'][k_] >= 1) / len(big) for k_ in used]
    m['motif_recurrence'] = float(np.average(rec, weights=[kinds[k_] for k_ in used])) if used else None    # share of parts carrying each used motif
    m['motif_recurrence_min'] = float(min(rec)) if used else None
    vec = [np.array([r['motifs'][k_] for k_ in MOTIFS], float) for r in big if r['marks'] >= 3]
    cs = [float(a @ b / (np.linalg.norm(a) * np.linalg.norm(b))) for i, a in enumerate(vec) for b in vec[i + 1:]]
    m['motif_mix_similarity'] = float(np.mean(cs)) if cs else None
    m['parts_with_marks'] = sum(1 for r in big if r['marks'] >= 1) / len(big)
    if mk:
        ml = np.array([d['length'] for d in mk]); mwid = np.array([d['width'] for d in mk]); H = math.sqrt(tot)
        m['mark_len_med_rel'] = float(np.median(ml) / H * 100); m['mark_width_med_rel'] = float(np.median(mwid) / H * 100)
        m['mark_len_med_u'] = float(np.median(ml)); m['mark_width_med_u'] = float(np.median(mwid))
        m['mark_size_logsd'] = float(np.log10([d['area'] for d in mk]).std())
        m['mark_single_share'] = float(np.mean([len(d['faces']) == 1 for d in mk])); m['mark_faces_med'] = float(np.median([len(d['faces']) for d in mk]))
        m['splinters_per_1000'] = 1000.0 * sum(1 for d in mk if len(d['faces']) == 1) / len(C)
        m['mark_contrast_med'] = float(np.median([abs(d['contrast']) for d in mk if 'accent' not in d['kind']] or [0]))
        lw = np.array([d['width'] for d in mk if 'line' in d['kind']])
        m['line_width_med_u'] = float(np.median(lw)) if len(lw) else None
        m['line_width_logsd'] = float(np.log10(np.maximum(lw, .1)).std()) if len(lw) > 2 else None
    # ---- focal detail against calm
    dens = np.array([r['mark_density'] for r in big]); bd = np.array([r['boundary_density'] for r in big])
    m['focal_mark_ratio'] = float(dens.max() / max(np.median(dens), 1e-9)) if len(mk) >= 8 and np.median(dens) > 0 else None
    m['focal_part_mark_share'] = float(max(r['marks'] for r in big) / max(len(mk), 1))
    m['focal_part_area_share'] = float(big[int(np.argmax([r['marks'] for r in big]))]['share'])
    m['focal_boundary_ratio'] = float(bd.max() / max(np.median(bd), 1e-9))
    m['calm_boundary_ratio'] = float(bd.min() / max(np.median(bd), 1e-9))
    m['focal_name'] = big[int(np.argmax(dens))]['name']
    srt = np.sort(dens); n = len(srt); m['mark_gini'] = float((2 * np.arange(1, n + 1) - n - 1) @ srt / max(n * srt.sum(), 1e-9))
    # ---- left-right symmetry of paint
    m.update(symmetry(sk))
    return dict(m=m, parts=rows, mats=mats, mat_of=mat_of, part=part, names=names, marks=mk)

def form_numbers(el, fold, change, same_mat, dl):
    """planes_toned  share of fold length (30 degrees or more) where the two planes carry different paint;
    flat_broken   share of flat edge length (under 12 degrees) that is a paint boundary;
    form_ratio    the first over the second: how much more often paint changes at a fold than inside a flat surface;
    plane_step    mean lightness difference across a fold, same material (0 where the paint does not change);
    flat_soft     share of flat edge length that is a SOFT boundary (same material, under 8 lightness): mottle."""
    sh, fl = fold >= 30, fold < 12
    if el[sh].sum() <= 0 or el[fl].sum() <= 0: return {}
    o = dict(planes_toned=float(el[sh & change].sum() / el[sh].sum()), flat_broken=float(el[fl & change].sum() / el[fl].sum()))
    o['form_ratio'] = o['planes_toned'] / max(o['flat_broken'], 0.02)
    o['plane_step'] = float((el * dl)[sh & same_mat].sum() / max(el[sh & same_mat].sum(), 1e-9))
    o['flat_soft'] = float(el[fl & change & same_mat & (dl < 8)].sum() / el[fl].sum())
    o['planes_same_mat'] = float(el[sh & change & same_mat].sum() / max(el[sh & same_mat].sum(), 1e-9))
    return o

def symmetry(sk):
    from scipy.spatial import cKDTree
    cen, area, C = sk['cen'], sk['area'], sk['C']
    V = sk['V'][np.unique(sk['F'])]; ext = (V.max(0) - V.min(0)).max()
    best = None
    for x0 in (0.0, float((V[:, 0].min() + V[:, 0].max()) / 2)):
        mir = cen.copy(); mir[:, 0] = 2 * x0 - mir[:, 0]
        d, i = cKDTree(cen).query(mir)
        ok = d <= 0.012 * ext
        geo = float(area[ok].sum() / area.sum())
        if best is None or geo > best[0]:
            same = C[i] == C; near = np.abs(np.array([hsl(c)[2] for c in C[i]]) - np.array([hsl(c)[2] for c in C])) <= 4
            best = (geo, float(area[ok & same].sum() / max(area[ok].sum(), 1e-9)), float(area[ok & near].sum() / max(area[ok].sum(), 1e-9)))
    return dict(mirror_geometry=best[0], mirror_paint_same=best[1], mirror_paint_near=best[2])

# ------------------------------------------------------------------------------------------------ pictures
def paint_view(sk, yaw=0.0, pitch=0.0, ppu=None, size=300, face_rgb=None, bg=(48, 43, 37), return_ids=False, all_faces=True):
    """Unlit paint picture by an own z-buffer. yaw 0 = seen from the front (-z side), 180 = from behind; pitch = degrees
    the camera looks down. Translucent faces are drawn too (as painted) when all_faces is set."""
    m = sk['mesh']; V = m['V']
    if all_faces:
        area = cmesh.face_geometry(m)[0]
        keep = (m['A'] != 255) & (m['T'] < 2) & (area > 0); F = m['F'][keep]; Cc = m['C'][keep]
        cols = TABLE[Cc].astype(np.uint8) if face_rgb is None else None
    else: F = sk['F']; Cc = sk['C']; cols = TABLE[Cc].astype(np.uint8)
    if face_rgb is not None: F = sk['F']; cols = np.asarray(face_rgb, np.uint8)
    t, p = math.radians(yaw), math.radians(pitch)
    f = np.array([math.sin(t), 0, math.cos(t)]); r = np.array([-math.cos(t), 0, math.sin(t)])
    fwd = f * math.cos(p) + np.array([0, 1, 0]) * math.sin(p); down = np.array([0, 1, 0]) * math.cos(p) - f * math.sin(p)
    used = np.unique(F); P = np.stack([V @ r, V @ down, V @ fwd], 1)
    lo, hi = P[used].min(0), P[used].max(0)
    s = ppu if ppu else (size - 12) / max(hi[0] - lo[0], hi[1] - lo[1], 1)
    W, H = int((hi[0] - lo[0]) * s) + 12, int((hi[1] - lo[1]) * s) + 12
    X = (P[:, 0] - lo[0]) * s + 6; Y = (P[:, 1] - lo[1]) * s + 6; Z = P[:, 2]
    img = np.zeros((H, W, 3), np.uint8); img[:] = bg; zb = np.full((H, W), np.inf); ids = np.full((H, W), -1, int)
    for k, tri in enumerate(F):
        x, y, z = X[tri], Y[tri], Z[tri]
        x0, x1 = max(int(math.floor(x.min())), 0), min(int(math.ceil(x.max())), W - 1); y0, y1 = max(int(math.floor(y.min())), 0), min(int(math.ceil(y.max())), H - 1)
        if x1 < x0 or y1 < y0: continue
        den = (y[1] - y[2]) * (x[0] - x[2]) + (x[2] - x[1]) * (y[0] - y[2])
        if abs(den) < 1e-9: continue
        gx, gy = np.meshgrid(np.arange(x0, x1 + 1) + 0.5, np.arange(y0, y1 + 1) + 0.5)
        a = ((y[1] - y[2]) * (gx - x[2]) + (x[2] - x[1]) * (gy - y[2])) / den
        b = ((y[2] - y[0]) * (gx - x[2]) + (x[0] - x[2]) * (gy - y[2])) / den
        c = 1 - a - b
        zz = a * z[0] + b * z[1] + c * z[2]
        ins = (a >= -1e-6) & (b >= -1e-6) & (c >= -1e-6) & (zz < zb[y0:y1 + 1, x0:x1 + 1])
        if ins.any():
            zb[y0:y1 + 1, x0:x1 + 1][ins] = zz[ins]; img[y0:y1 + 1, x0:x1 + 1][ins] = cols[k]; ids[y0:y1 + 1, x0:x1 + 1][ins] = k
    return (img, ids) if return_ids else img

PART_TINT = [(230, 80, 80), (80, 200, 90), (90, 130, 240), (235, 200, 70), (200, 90, 220), (70, 210, 210), (240, 150, 60), (160, 160, 160), (120, 90, 60), (250, 250, 250)]

def jsonable(o):
    if isinstance(o, dict): return {str(k): jsonable(v) for k, v in o.items()}
    if isinstance(o, (list, tuple)): return [jsonable(v) for v in o]
    if isinstance(o, np.ndarray): return o.tolist()
    if isinstance(o, (np.floating,)): return float(o)
    if isinstance(o, (np.integer,)): return int(o)
    return o
