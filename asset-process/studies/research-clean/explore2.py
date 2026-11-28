"""Exploration 2: structure of the lit picture and of the lit mesh: is tone variation concentrated on a few crisp
lines (clean) or spread as many weak edges (splotch)?  -> data/explore2.json"""
import sys, pathlib, json, math, collections
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np, cv2
import cleanlib as cl
from explore import REAL, OURS, all_entries, same_mat
HERE = pathlib.Path(__file__).resolve().parent

def image_structure(c, px=420):
    skin = c['drawn'] & (c['A'] == 0)
    used = np.unique(c['F'][skin]); hgt = np.ptp(c['V'][used][:, 1])
    G = []; calm = []; body = 0
    out = {}
    comp_sizes = []
    for yaw in (25, 205):
        img, ids = cl.view(c, yaw, 8, ppu=px / hgt, faces=np.nonzero(skin)[0])
        g = cl.grey(img).astype(np.float32)
        m = (ids >= 0).astype(np.uint8)
        inner = cv2.erode(m, np.ones((5, 5), np.uint8)) > 0
        gx = cv2.Sobel(g, cv2.CV_32F, 1, 0, ksize=3) / 8; gy = cv2.Sobel(g, cv2.CV_32F, 0, 1, ksize=3) / 8
        mag = np.hypot(gx, gy)
        G.append(mag[inner]); body += int(inner.sum())
        # calm regions: connected pixels with gradient under 1.5 grey levels per pixel
        cm = (inner & (mag < 1.5)).astype(np.uint8)
        n, lab, stats, _ = cv2.connectedComponentsWithStats(cm, connectivity=4)
        comp_sizes.append(stats[1:, cv2.CC_STAT_AREA] / inner.sum())
    g = np.concatenate(G); s = np.concatenate(comp_sizes)
    out['grad_mean'] = float(g.mean())
    out['edge_weak'] = float(((g >= 1.5) & (g < 5)).mean()); out['edge_mid'] = float(((g >= 5) & (g < 12)).mean()); out['edge_strong'] = float((g >= 12).mean())
    out['weak_over_strong'] = out['edge_weak'] / max(out['edge_strong'], 1e-4)
    out['grad_in_strong'] = float(g[g >= 8].sum() / max(g.sum(), 1e-9))        # share of all tone change carried by crisp lines
    gs = np.sort(g); n = len(gs); out['grad_gini'] = float((2 * np.arange(1, n + 1) - n - 1) @ gs / (n * gs.sum()))
    o = np.argsort(-s); cum = np.cumsum(s[o]); tot = cum[-1] if len(cum) else 1
    out['calm_share'] = float(s.sum() / 2)
    out['calm_region_wmed'] = float(s[o][np.searchsorted(cum, 0.5 * tot)]) if len(s) else 0.0   # half the calm area lies in regions at least this big (share of body)
    out['calm_regions_per_body'] = float((s > 0.0005).sum() / 2)
    out['calm_small_share'] = float(s[s < 0.004].sum() / max(s.sum(), 1e-9))
    return out

def mesh_structure(c):
    """Planes of the lit mesh: connected faces of one paint whose shared edge is not a visible line.
    A shared edge is a LINE if the paints differ, or the lit lightness has a kink there (the light gradient changes
    by more than a threshold across it)."""
    cl.topology(c)
    F, C, V, LC = c['F'], c['C'], c['V'], c['LC'].astype(float); area = c['area']
    skin = c['drawn'] & (c['A'] == 0); tot = area[skin].sum()
    used = np.unique(F[skin]); hgt = np.ptp(V[used][:, 1])
    out = {}
    # per-face corner range of lit lightness (how much light gradient one face carries)
    rng = LC.max(1) - LC.min(1)
    out['face_range_med'] = float(np.median(rng[skin])); out['face_range_p90'] = float(np.quantile(rng[skin], .9))
    out['face_range_areaw'] = float((rng * area)[skin].sum() / tot)
    # gradient of lit lightness per face as a 3D vector (lightness per unit), then the kink across each edge
    a, b, d = V[F[:, 0]], V[F[:, 1]], V[F[:, 2]]
    e1, e2 = b - a, d - a
    n = np.cross(e1, e2); nn = (n * n).sum(1); nn[nn == 0] = 1
    l0, l1, l2 = LC[:, 0], LC[:, 1], LC[:, 2]
    grad = ((l1 - l0)[:, None] * np.cross(e2, n) + (l2 - l0)[:, None] * np.cross(n, e1)) / nn[:, None]
    ea, eb, el, ev, dih = c['e_a'], c['e_b'], c['e_len'], c['e_v'], c['e_dih']
    both = skin[ea] & skin[eb]; ea, eb, el, ev, dih = ea[both], eb[both], el[both], ev[both], dih[both]
    # kink: change of the gradient component ACROSS the edge, times a reference length (1.5% of height) = lightness steps
    ed = V[ev[:, 1]] - V[ev[:, 0]]; ed /= np.maximum(np.linalg.norm(ed, axis=1), 1e-9)[:, None]
    def across(i):
        t = np.cross(c['unit'][i], ed); return t
    ta = np.cross(c['unit'][ea], ed); tb = np.cross(c['unit'][eb], ed)
    # orient both tangents to point from face a to face b
    sa = np.sign(((c['cen'][eb] - c['cen'][ea]) * ta).sum(1)); sb = np.sign(((c['cen'][eb] - c['cen'][ea]) * tb).sum(1))
    ga = (grad[ea] * ta).sum(1) * sa; gb = (grad[eb] * tb).sum(1) * sb
    ref = 0.02 * hgt
    kink = np.abs(ga - gb) * ref
    paint_step = np.abs((C[ea] & 127).astype(float) - (C[eb] & 127)) * (c['LN'][ev].mean(1) / 128)
    samecol = C[ea] == C[eb]
    sm = np.array([same_mat(int(x), int(y)) for x, y in zip(C[ea], C[eb])])
    out['kink_med'] = float(np.median(kink)); out['kink_p90'] = float(np.quantile(kink, .9))
    for t in (2, 4, 8):
        out[f'kink_over{t}'] = float(el[kink >= t].sum() / el.sum())
    out['kink_2to8'] = out['kink_over2'] - out['kink_over8']
    out['kink_weak_over_strong'] = out['kink_2to8'] / max(out['kink_over8'], 1e-4)
    line = (~samecol) | (kink >= 4)
    parent = {int(i): int(i) for i in np.nonzero(skin)[0]}
    def find(x):
        while parent[x] != x: parent[x] = parent[parent[x]]; x = parent[x]
        return x
    for x, y, ln_ in zip(ea, eb, line):
        if not ln_: parent[find(int(x))] = find(int(y))
    pa = collections.Counter(); pf = collections.Counter()
    for i in parent: r = find(i); pa[r] += area[i]; pf[r] += 1
    s = np.array(sorted(pa.values(), reverse=True)) / tot; cum = np.cumsum(s)
    out['plane_wmed'] = float(s[np.searchsorted(cum, 0.5)])                 # half the skin lies in planes at least this big (share of skin)
    out['planes_per100'] = 100 * len(pa) / skin.sum()
    out['plane_single_area'] = float(sum(pa[r] for r in pa if pf[r] == 1) / tot)
    out['plane_small_area'] = float(sum(pa[r] for r in pa if pa[r] < 0.002 * tot) / tot)
    out['line_share'] = float(el[line].sum() / el.sum())
    out['lines_paint_only'] = float(el[(~samecol) & (kink < 4)].sum() / max(el[line].sum(), 1e-9))
    out['lines_light_only'] = float(el[samecol & (kink >= 4)].sum() / max(el[line].sum(), 1e-9))
    # lit jump across the edge: the paint step as drawn
    out['paintline_soft'] = float(el[(~samecol) & sm & (paint_step < 3)].sum() / max(el[(~samecol) & sm].sum(), 1e-9))
    out['litstep_med'] = float(np.median(paint_step[(~samecol) & sm])) if ((~samecol) & sm).any() else 0.0
    return out

if __name__ == '__main__':
    store = HERE / 'data' / 'explore2.json'
    rows = json.loads(store.read_text()) if store.exists() and '--refresh' not in sys.argv else {}
    for key, entry, name, st in all_entries():
        if key in rows: continue
        c = cl.load(entry); r = image_structure(c); r.update(mesh_structure(c)); r['name'] = name; r['style'] = st; rows[key] = r
        print('measured', name, file=sys.stderr); store.write_text(json.dumps(rows, indent=0))
    keys = [k for k in next(iter(rows.values())) if k not in ('name', 'style')]
    real = [r for r in rows.values() if r['style'] != 'o']
    oursk = ['watcher-2', 'watcher-repaint', 'star-hound-3', 'warden-general-3']
    print(f"{'measure':24s} {'real min':>8s} {'p10':>7s} {'med':>7s} {'p90':>7s} {'max':>8s} | {'smooth':>7s} {'detail':>7s} | {'Mino':>7s} {'Tekton':>7s} | {'w2':>7s} {'w2-rep':>7s} {'hound3':>7s} {'WG3':>7s} | out")
    for k in keys:
        v = np.array([r[k] for r in real], float)
        s_ = np.median([r[k] for r in real if r['style'] == 's']); d_ = np.median([r[k] for r in real if r['style'] == 'd'])
        o = [rows[q][k] for q in oursk]
        flags = ''.join('<' if x < v.min() else '>' if x > v.max() else ('(' if x < np.quantile(v, .1) else ')' if x > np.quantile(v, .9) else '.') for x in o)
        print(f"{k:24s} {v.min():8.3f} {np.quantile(v, .1):7.3f} {np.median(v):7.3f} {np.quantile(v, .9):7.3f} {v.max():8.3f} | {s_:7.3f} {d_:7.3f} | {rows['12812'][k]:7.3f} {rows['7541'][k]:7.3f} | " + ' '.join(f'{x:7.3f}' for x in o) + ' | ' + flags)
