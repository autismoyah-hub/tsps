"""measures.py: every measure of the helm2 study, per helm -> data/measures.json
usage: measures.py [key ...]     (default: all of h2.ORDER; results are merged into the json)

PIXEL measures come from the client's own picture of the helm IN ITS OUTFIT (tools/pose.py, actor draw), bind pose,
head aimed, at game size (zoom 300: a unit is 1.7 px) and close (zoom 130: 3.9 px), from yaw 0 front, 256 front
quarter, 512 side (the unlit side), 1024 back, 1536 other side (lit). The helm's pixels = pixels that differ between
the outfit with and without the helm and are not background. MESH measures come from the worn mesh.
"""
import sys, pathlib, json, collections
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
from scipy import ndimage
from PIL import Image, ImageDraw
import h2
from h2 import L, deflib
sys.path.insert(0, str(h2.ROOT / 'studies' / 'research-fit'))
import fit as FIT, fitlib

YAWS = (0, 256, 512, 1024, 1536)
VN = {0: 'front', 256: 'quarter', 512: 'side', 1024: 'back', 1536: 'litside'}
DATA = h2.HERE / 'data'; DATA.mkdir(exist_ok=True)
SKIN = lambda h, s, l: (h == 4) & (s == 3)


# ------------------------------------------------------------------------------------------------ pixels
def pictures(keys):
    """-> {(key, zoomname, yaw): (with, without)} raw cells; zoomname 'g' (game) or 'c' (close)"""
    es = []
    for k in keys: es += [h2.entry(k), h2.entry(k, helm=False)]
    out = {}
    for zn, zoom, cell in (('g', 300, (170, 170)), ('c', 130, (320, 340)), ('f', 300, (240, 450))):
        for yaw in YAWS:
            if zn == 'f' and yaw not in (0, 1024): continue
            a = h2.draw(f'm-{zn}-{yaw}', es, yaw=yaw, zoom=zoom, cell=cell, aim=None if zn == 'f' else 'head')
            cs = h2.cells(a, len(es))
            for i, k in enumerate(keys): out[k, zn, yaw] = (cs[2 * i], cs[2 * i + 1])
    return out


def helm_cell(w, wo):
    bg = w[2, 2]
    M = h2.helm_mask(w, wo)
    lab, n = ndimage.label(M, np.ones((3, 3)))
    if n > 1:                      # keep the pieces of real size (drops stray differing pixels on the body)
        sz = ndimage.sum(M, lab, range(1, n + 1)); keep = [i + 1 for i, s in enumerate(sz) if s >= max(6, 0.02 * sz.max())]
        M = np.isin(lab, keep)
    c = w.copy(); c[~M] = bg
    return c, M, bg


def scan_bands(G, M, axis, fracs=(0.3, 0.4, 0.5, 0.6, 0.7), T=12):
    """mean number of tone steps of T grey or more met along scan lines through the helm (axis 0: down the helm,
    counting level bands; axis 1: across it, counting upright bands), between 20% and 85% of its extent."""
    ys, xs = np.nonzero(M)
    y0, y1, x0, x1 = ys.min(), ys.max(), xs.min(), xs.max()
    out = []
    for f in fracs:
        if axis == 0:
            x = int(round(x0 + f * (x1 - x0))); g = G[:, x]; m = M[:, x]
        else:
            y = int(round(y0 + f * (y1 - y0))); g = G[y, :]; m = M[y, :]
        idx = np.nonzero(m)[0]
        if len(idx) < 6: continue
        lo, hi = idx.min(), idx.max(); a = int(lo + 0.15 * (hi - lo)); b = int(lo + 0.9 * (hi - lo))
        seg = g[a:b + 1]; ok = m[a:b + 1]
        d = np.abs(np.diff(seg)); both = ok[1:] & ok[:-1]
        # merge consecutive step pixels into one step
        st = (d >= T) & both
        out.append(int((st[1:] & ~st[:-1]).sum() + st[0]))
    return float(np.mean(out)) if out else 0.0


def pix(key, P):
    r = {}
    for zn in ('g', 'c'):
        for yaw in YAWS:
            w, wo = P[key, zn, yaw]
            c, M, bg = helm_cell(w, wo)
            G = h2.grey(w); bgG = float(bg.sum() / 3.0)
            v = VN[yaw]; t = f'{zn}_{v}_'
            area = int(M.sum()); r[t + 'area'] = area
            if area < 30: continue
            g = G[M]
            for q in (5, 10, 50, 90, 95): r[t + f'p{q:02d}'] = round(float(np.percentile(g, q)), 1)
            r[t + 'range'] = round(r[t + 'p95'] - r[t + 'p05'], 1)
            r[t + 'dark20'] = round(float((g < 20).mean()), 3)
            r[t + 'near_bg'] = round(float((np.abs(g - bgG) < 12).mean()), 3)        # helm pixels within 12 grey of the ground
            d = deflib.analyse(c, bg)
            for k in ('step', 'step40', 'step_level', 'step_upright', 'lines', 'line_light', 'line_dark', 'line_mat', 'regions', 'reg_big', 'contrast', 'step_mat', 'step_dark', 'step_tone'):
                r[t + k] = d[k]
            r[t + 'level_ratio'] = round(d['step_level'] / max(d['step_upright'], 0.01), 2)
            r[t + 'reg_per1000'] = d['reg_per1000']
            # outline against the ground: helm pixels with a background neighbour
            fig = np.abs(w - bg).sum(2) > 0
            nb = ndimage.binary_dilation(~fig, np.ones((3, 3))) & M
            if nb.sum() >= 5:
                e = np.abs(G[nb] - bgG)
                r[t + 'rim_contrast'] = round(float(np.median(e)), 1); r[t + 'rim_lost'] = round(float((e < 12).mean()), 3)
                r[t + 'rim_n'] = int(nb.sum())
            # the face void: largest patch of near-black (grey < 16) and how light the helm is right round it
            dk = M & (G < 16)
            lab, n = ndimage.label(dk)
            if n:
                sz = ndimage.sum(dk, lab, range(1, n + 1)); big = lab == (1 + int(np.argmax(sz)))
                ring = ndimage.binary_dilation(big, iterations=4 if zn == 'c' else 2) & M & ~dk
                r[t + 'void_big'] = round(float(big.sum() / area), 3)
                r[t + 'void_all'] = round(float(dk.sum() / area), 3)
                r[t + 'void_ring'] = round(float(np.median(G[ring]) - np.median(G[big])), 1) if ring.sum() >= 4 else 0.0
                r[t + 'void_n'] = int((sz >= max(4, 0.004 * area)).sum())
            else:
                r[t + 'void_big'] = r[t + 'void_all'] = r[t + 'void_ring'] = 0.0; r[t + 'void_n'] = 0
            r[t + 'bands_down'] = round(scan_bands(G, M, 0), 2)
            r[t + 'bands_across'] = round(scan_bands(G, M, 1), 2)
            r[t + 'band_ratio'] = round(r[t + 'bands_down'] / max(r[t + 'bands_across'], 0.2), 2)
            # materials in the picture
            hh, ss, ll, ok = L.hsl_of(w[M])
            lit = ok & (ll >= 10) & ~SKIN(hh, ss, ll)
            if lit.sum() > 20:
                cnt = collections.Counter(zip((hh[lit] // 2 * 2 if False else hh[lit]).tolist(), np.minimum(ss[lit], 9).tolist()))
                # material families: grey (s <= 1) is one family whatever its hue
                fam = np.where(ss <= 1, -1, hh)
                famc = collections.Counter(fam[lit].tolist()); main = famc.most_common(1)[0][0]
                ismain = lit & (fam == main)
                r[t + 'second_share'] = round(float(1 - ismain.sum() / lit.sum()), 3)
                r[t + 'main_L'] = float(np.median(ll[ismain])); r[t + 'main_S'] = float(np.median(ss[ismain]))
                sec = lit & ~ismain
                r[t + 'second_L'] = float(np.median(ll[sec])) if sec.sum() >= 3 else None
                r[t + 'families'] = len([1 for f, n in famc.items() if n >= 0.01 * lit.sum()])
            # the body next to the helm (game size only): the 60 rows under the helm, unchanged figure pixels, no skin
            if zn == 'g':
                ys = np.nonzero(M)[0]; yb = ys.max()
                zone = np.zeros_like(M); zone[yb - 6:yb + 40] = True
                B = zone & fig & ~M
                hb, sb, lb, okb = L.hsl_of(w[B])
                keep = ~(okb & SKIN(hb, sb, lb))
                gb = G[B][keep]
                if len(gb) > 30:
                    r[t + 'body_p50'] = round(float(np.median(gb)), 1)
                    r[t + 'helm_minus_body'] = round(r[t + 'p50'] - r[t + 'body_p50'], 1)
                    r[t + 'body_range'] = round(float(np.percentile(gb, 95) - np.percentile(gb, 5)), 1)
                    r[t + 'range_vs_body'] = round(r[t + 'range'] / max(r[t + 'body_range'], 1), 2)
                    r[t + 'p95_minus_body_p95'] = round(r[t + 'p95'] - float(np.percentile(gb, 95)), 1)
                    r[t + 'skin_share_body'] = round(float(1 - keep.mean()), 2)
    # the helm's share of the whole figure, and its definition against the body's (whole figure, game size)
    for yaw in (0, 1024):
        w, wo = P[key, 'f', yaw]; c, M, bg = helm_cell(w, wo)
        fig = np.abs(w - bg).sum(2) > 0
        v = VN[yaw]
        r[f'f_{v}_helm_share'] = round(float(M.sum() / fig.sum()), 3)
        cb = w.copy(); cb[M] = bg
        db = deflib.analyse(cb, bg); dh = deflib.analyse(c, bg)
        if db and dh:
            r[f'f_{v}_step_helm_over_body'] = round(dh['step'] / max(db['step'], 0.1), 2)
            r[f'f_{v}_lines_helm_over_body'] = round(dh['lines'] / max(db['lines'], 0.1), 2)
            r[f'f_{v}_body_step'] = db['step']; r[f'f_{v}_body_lines'] = db['lines']; r[f'f_{v}_body_level'] = db['step_level']
            r[f'f_{v}_body_p50'] = db['p50']
        xs = np.nonzero(fig.any(0))[0]; hx = np.nonzero(M.any(0))[0]; hy = np.nonzero(M.any(1))[0]; fy = np.nonzero(fig.any(1))[0]
        r[f'f_{v}_helm_w_over_fig_w'] = round(float(len(hx) / len(xs)), 3)
        r[f'f_{v}_helm_h_over_fig_h'] = round(float((hy.max() - hy.min() + 1) / (fy.max() - fy.min() + 1)), 3)
    # means over the five views
    for zn in ('g', 'c'):
        for k in ('step', 'lines', 'regions', 'range', 'p50', 'rim_contrast', 'rim_lost', 'level_ratio', 'band_ratio', 'bands_down', 'near_bg', 'line_light', 'step40'):
            vals = [r.get(f'{zn}_{VN[y]}_{k}') for y in YAWS]
            vals = [x for x in vals if x is not None]
            if vals:
                r[f'{zn}_all_{k}'] = round(float(np.mean(vals)), 2); r[f'{zn}_min_{k}'] = round(float(np.min(vals)), 2)
        # side and back against the front: does the helm go dead when turned?
        for k in ('step', 'range', 'lines'):
            fr = r.get(f'{zn}_front_{k}'); o = [r.get(f'{zn}_{v}_{k}') for v in ('side', 'back')]
            if fr and None not in o: r[f'{zn}_turn_{k}'] = round(float(np.mean(o) / max(fr, 0.01)), 2)
    return r


# ------------------------------------------------------------------------------------------------ mesh
S = 4.0        # raster pixels per unit for silhouettes


def raster(T, ax, box):
    """orthographic silhouette of triangles T [n,3,3] seen along x ('side': z across) or z ('front': x across)"""
    (u0, u1), (y0, y1) = box
    W, Hh = int((u1 - u0) * S) + 1, int((y1 - y0) * S) + 1
    im = Image.new('1', (W, Hh), 0); d = ImageDraw.Draw(im)
    ui = 0 if ax == 'front' else 2
    for t in T:
        d.polygon([((p[ui] - u0) * S, (y1 - p[1]) * S) for p in t], fill=1, outline=1)
    return np.asarray(im).astype(bool)


def outline(key, mh):
    r = {}
    T = L.tris(mh['V'], mh['F']); HT = fitlib.head_tris()
    for ax in ('front', 'side'):
        box = ((-45, 45), (155, 230))
        A = raster(T, ax, box); Hd = raster(HT, ax, box)
        # keep the head above the collar only (the kits' neck goes down to 164)
        px = 1.0 / S
        r[f'sil_{ax}_area'] = round(float(A.sum() * px * px), 1)
        r[f'sil_{ax}_over_head'] = round(float(A.sum() / Hd.sum()), 2)
        near = ndimage.binary_dilation(Hd, iterations=int(3 * S))          # within 3 units of the bare head: the shell
        sig = A & ~near
        r[f'sig_{ax}_area'] = round(float(sig.sum() * px * px), 1)
        r[f'sig_{ax}_share'] = round(float(sig.sum() / max(A.sum(), 1)), 3)
        dist = ndimage.distance_transform_edt(~Hd) * px
        r[f'sig_{ax}_reach'] = round(float(dist[A].max()), 1) if A.any() else 0.0
        if sig.sum() > 10:
            per = (sig & ~ndimage.binary_erosion(sig)).sum() * px
            r[f'sig_{ax}_thick'] = round(float(2 * sig.sum() * px * px / max(per, 1e-6)), 2)           # mean width, units
            r[f'sig_{ax}_thick_px'] = round(r[f'sig_{ax}_thick'] * 1.7, 2)                           # at game size
            # share of the signature thick enough to survive at game size: wider than 2 px (1.2 units) / 3 px (1.8)
            for name, wd in (('w2', 1.2), ('w3', 1.8), ('w5', 3.0)):
                it = max(1, int(round(wd * S / 2)))
                op = ndimage.binary_dilation(ndimage.binary_erosion(sig, iterations=it), iterations=it) & sig
                r[f'sig_{ax}_{name}'] = round(float(op.sum() / sig.sum()), 3)
                if name == 'w3': r[f'sig_{ax}_solid_area'] = round(float(op.sum() * px * px), 1)
            lab, n = ndimage.label(sig); sz = ndimage.sum(sig, lab, range(1, n + 1)) * px * px
            r[f'sig_{ax}_parts'] = int((sz >= 2.0).sum()); r[f'sig_{ax}_biggest'] = round(float(sz.max()), 1)
        else:
            for k in ('thick', 'thick_px', 'w2', 'w3', 'w5', 'solid_area', 'biggest'): r[f'sig_{ax}_{k}'] = 0.0
            r[f'sig_{ax}_parts'] = 0
        # solidity: silhouette area / convex hull area
        ys, xs = np.nonzero(A)
        from scipy.spatial import ConvexHull
        hull = ConvexHull(np.c_[xs, ys]); r[f'sil_{ax}_solidity'] = round(float(A.sum() / hull.volume), 3)
        r[f'sil_{ax}_w'] = round(float((xs.max() - xs.min()) * px), 1); r[f'sil_{ax}_h'] = round(float((ys.max() - ys.min()) * px), 1)
    r['sig_min_area'] = min(r['sig_front_area'], r['sig_side_area'])
    r['sig_min_solid'] = min(r['sig_front_solid_area'], r['sig_side_solid_area'])
    r['sig_min_thick_px'] = min(r['sig_front_thick_px'], r['sig_side_thick_px'])
    return r


def edges_of(mh):
    V = mh['V']; F = [f for f in mh['F'] if len(set(f)) == 3]
    idx = [i for i, f in enumerate(mh['F']) if len(set(f)) == 3]
    # weld by position
    key = {}; wid = []
    for p in V:
        k = tuple(np.round(p, 3)); wid.append(key.setdefault(k, len(key)))
    T = np.array([[V[a], V[b], V[c]] for a, b, c in F], float)
    n = np.cross(T[:, 1] - T[:, 0], T[:, 2] - T[:, 0]); area = np.linalg.norm(n, axis=1) / 2
    n = n / np.maximum(np.linalg.norm(n, axis=1)[:, None], 1e-9)
    ed = collections.defaultdict(list)
    for i, f in enumerate(F):
        w = [wid[v] for v in f]
        for a, b in ((0, 1), (1, 2), (2, 0)):
            ed[tuple(sorted((w[a], w[b])))].append((i, np.linalg.norm(V[f[a]] - V[f[b]])))
    col = [mh['col'][i] for i in idx]
    return T, n, area, ed, col, idx


def mesh_measures(key, mh, mb):
    r = {}
    T, n, area, ed, col, idx = edges_of(mh)
    tot = area.sum(); r['faces'] = len(T); r['surface'] = round(float(tot), 0)
    colA = np.array(col)
    shell = np.array([not fitlib.wearer_face(mh['col'][i], mh['raw'][i]) for i in idx])
    # folds and painted boundaries
    f45 = hard = firm = totl = painted = folded_plain = painted_flat = 0.0
    adj = collections.defaultdict(list)
    for e, fs in ed.items():
        if len(fs) != 2: continue
        (i, ln), (j, _) = fs
        if not (shell[i] and shell[j]): continue
        ang = np.degrees(np.arccos(np.clip(n[i] @ n[j], -1, 1)))
        diff = tuple(col[i]) != tuple(col[j]); samemat = (col[i][0], col[i][1]) == (col[j][0], col[j][1]) or (col[i][1] <= 1 and col[j][1] <= 1)
        dl = abs(col[i][2] - col[j][2])
        totl += ln
        if ang > 45: f45 += ln
        if samemat:
            if ang >= 50: hard += ln
            elif ang >= 30: firm += ln
        if diff and (not samemat or dl >= 12): painted += ln
        if ang >= 30 and not diff: folded_plain += ln
        if diff and ang <= 15: painted_flat += ln
        if (not diff) and ang < 20: adj[i].append(j); adj[j].append(i)
    sq = np.sqrt(area[shell].sum())
    r['fold45_share'] = round(float(f45 / totl), 3)
    r['hard_over_firm'] = round(float(hard / max(firm, 1e-6)), 2); r['firm_share'] = round(float(firm / totl), 3)
    r['painted_per_sqrt'] = round(float(painted / sq), 1)
    r['plainfold_per_sqrt'] = round(float(folded_plain / sq), 1)
    r['paintflat_per_sqrt'] = round(float(painted_flat / sq), 1)
    r['paintflat_over_painted'] = round(float(painted_flat / max(painted, 1e-6)), 2)
    # planes: faces of one paint joined across edges under 20 degrees
    seen = np.zeros(len(T), bool); planes = []
    for s0 in range(len(T)):
        if seen[s0] or not shell[s0]: continue
        st = [s0]; seen[s0] = True; a = 0.0; cnt = 0
        while st:
            i = st.pop(); a += area[i]; cnt += 1
            for j in adj[i]:
                if not seen[j]: seen[j] = True; st.append(j)
        planes.append((a, cnt))
    pa = np.array(sorted([p[0] for p in planes], reverse=True)); sa = pa.sum()
    r['planes'] = len(pa); r['plane_big'] = round(float(pa[0] / sa), 3)
    r['planes_for_80'] = int(np.searchsorted(np.cumsum(pa) / sa, 0.8) + 1)
    r['plane_top5'] = round(float(pa[:5].sum() / sa), 3)
    r['small_plane_share'] = round(float(pa[pa < 0.01 * sa].sum() / sa), 3)
    r['planes_over_2pc'] = int((pa >= 0.02 * sa).sum())
    r['faces_per_plane'] = round(float(shell.sum() / len(pa)), 2)
    # paint: families, lightness spread (area weighted) on the shell
    fam = np.where(colA[:, 1] <= 1, -1, colA[:, 0])
    w = area * shell
    fams = collections.Counter()
    for f, a in zip(fam.tolist(), w.tolist()): fams[f] += a
    main = fams.most_common(1)[0][0]; ism = (fam == main) & shell
    r['paint_main_share'] = round(float(w[ism].sum() / w.sum()), 3)
    r['paint_second_share'] = round(1 - r['paint_main_share'], 3)
    def wq(v, ww, q):
        o = np.argsort(v); c = np.cumsum(ww[o]) / ww.sum(); return float(v[o][np.searchsorted(c, q / 100.0)])
    Lm = colA[ism][:, 2].astype(float); wm = area[ism]
    for q in (5, 25, 50, 75, 95): r[f'paint_L{q:02d}'] = wq(Lm, wm, q)
    r['paint_Lrange'] = r['paint_L95'] - r['paint_L05']
    r['paint_main_hue'] = int(np.bincount(colA[ism][:, 0], weights=wm).argmax()); r['paint_main_sat'] = int(np.bincount(colA[ism][:, 1], weights=wm).argmax())
    r['paint_tones_main'] = len(set(colA[ism][:, 2].tolist()))
    sec = shell & ~ism
    if area[sec].sum() > 0:
        r['paint_second_L'] = wq(colA[sec][:, 2].astype(float), area[sec], 50); r['second_minus_main_L'] = r['paint_second_L'] - r['paint_L50']
        r['paint_second_sat'] = int(np.bincount(colA[sec][:, 1], weights=area[sec]).argmax())
    r['light_share'] = round(float(w[(colA[:, 2] >= r['paint_L50'] + 20) & shell].sum() / w.sum()), 3)     # surface 20+ lighter than the median steel
    r['dark_share'] = round(float(w[(colA[:, 2] <= 8) & shell].sum() / w.sum()), 3)
    # the body's paint for the match
    Tb, nb, ab, edb, colb, idxb = edges_of(mb); cb = np.array(colb)
    skin = (cb[:, 0] == 4) & (cb[:, 1] == 3)
    famb = np.where(cb[:, 1] <= 1, -1, cb[:, 0]); fb = collections.Counter()
    for f, a, sk in zip(famb.tolist(), ab.tolist(), skin.tolist()):
        if not sk: fb[f] += a
    mainb = fb.most_common(1)[0][0]; isb = (famb == mainb) & ~skin
    r['body_L50'] = wq(cb[isb][:, 2].astype(float), ab[isb], 50); r['body_L95'] = wq(cb[isb][:, 2].astype(float), ab[isb], 95); r['body_L05'] = wq(cb[isb][:, 2].astype(float), ab[isb], 5)
    r['helm_L50_minus_body'] = r['paint_L50'] - r['body_L50']
    r['helm_L95_minus_body'] = r['paint_L95'] - r['body_L95']
    r['body_main_hue'] = int(np.bincount(cb[isb][:, 0], weights=ab[isb]).argmax()); r['body_main_sat'] = int(np.bincount(cb[isb][:, 1], weights=ab[isb]).argmax())
    r['same_family_as_body'] = int((main == mainb) or (r['paint_main_hue'] == r['body_main_hue'] and abs(r['paint_main_sat'] - r['body_main_sat']) <= 1))
    r['body_second_share'] = round(float(1 - fb[mainb] / sum(fb.values())), 3)
    r['second_share_minus_body'] = round(r['paint_second_share'] - r['body_second_share'], 3)
    # where faces are spent
    mid = T.mean(1)
    r['faces_face'] = round(float(np.mean((mid[:, 2] < -12) & (mid[:, 1] < 195))), 2)
    r['faces_back'] = round(float(np.mean((mid[:, 2] > 0) & (mid[:, 1] < 195))), 2)
    r['faces_above'] = round(float(np.mean(mid[:, 1] >= 195)), 2)
    return r


def fit_measures(key):
    h = h2.H[key]
    kind, a = h['mesh']
    if kind == 'real':
        d, rows, cf, f = FIT.check(f'item:{a}', quiet=True)
    else:
        bk, ba = h['body']
        d, rows, cf, f = FIT.check(a, h['spec'], f'item:{ba}' if bk == 'real' else ba, quiet=True)
    out = {f'fit_{k}': d.get(k) for k in ('so_med', 'wid', 'hgt', 'dep', 'w_skull', 'd_skull', 'above', 'fore', 'helm_to_shoulder', 'cover', 'chin_overhang')}
    out['fit_core_fails'] = cf
    return out


if __name__ == '__main__':
    keys = sys.argv[1:] or h2.ORDER
    P = pictures(keys)
    f = DATA / 'measures.json'
    allr = json.load(open(f)) if f.exists() else {}
    for k in keys:
        mh = h2.mesh(k); mb = h2.body_mesh(k)
        r = dict(group=h2.H[k]['group'], name=h2.H[k]['name'])
        r.update(pix(k, P)); r.update(outline(k, mh)); r.update(mesh_measures(k, mh, mb)); r.update(fit_measures(k))
        allr[k] = r
        print(k, len(r), 'measures')
    json.dump(allr, open(f, 'w'), indent=1)
