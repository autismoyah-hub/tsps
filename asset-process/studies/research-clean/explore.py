"""Exploration: a broad set of candidate measures for 'clean / smooth versus splotchy' on real creatures and ours.
usage: explore.py [--refresh]   -> data/explore.json and a printed table with separation."""
import sys, pathlib, json, math, collections
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
import cleanlib as cl
sys.path.insert(0, str(cl.B / 'research-paint'))
import paintlib
HERE = pathlib.Path(__file__).resolve().parent
W = cl.B

REAL = {  # id: (name, style by LOOKING: s = smooth, d = detailed)
    12812: ('Minotaur', 's'), 7541: ('Tekton', 'd'), 13599: ('Tormented Demon', 'd'), 9293: ('Basilisk Knight', 's'), 2215: ('General Graardor', 's'),
    3129: ("K'ril Tsutsaroth", 's'), 5862: ('Cerberus', 'd'), 11278: ('Nex', 'd'), 12821: ('Sol Heredit', 'd'), 12223: ('Vardorvis', 'd'),
    11068: ('Lynx', 's'), 12976: ('Jaguar', 's'), 104: ('Hellhound', 's'), 2025: ('Greater demon', 's'), 7144: ('Demonic gorilla', 's'),
    11778: ('Ba-Ba', 'd'), 14176: ('Yama', 'd'), 15625: ('Bull', 's'), 7286: ('Skotizo', 'd'), 6609: ('Callisto', 's'), 7849: ('Dusk', 'd'),
    8061: ('Vorkath', 'd'), 11789: ('Akkha', 'd'), 12191: ('Duke Sucellus', 'd'),
}
OURS = {
    'watcher-2': W / 'creature-stone-watcher-2' / 'creature-stone-watcher-2.json',
    'watcher-repaint': W / 'research-paint' / 'repaint' / 'watcher-2-repaint.json',
    'star-hound-3': W / 'creature-star-hound-3' / 'creature-star-hound-3.json',
    'warden-general-3': W / 'boss-warden-general-3' / 'boss-warden-general-3.json',
}

def same_mat(a, b):
    return a == b or paintlib.near(a, b) is not None or paintlib.near(b, a) is not None

def ring_sign_changes(c, thr=8.0):
    """Per interior vertex: number of sign changes of the dihedral (edges with |angle| >= thr) going round its ring."""
    F = c['F']; ekey = {}
    for k, (a, b) in enumerate(c['e_v']): ekey[(int(a), int(b))] = k
    out = {}
    for v, fs in c['vf'].items():
        if len(fs) < 4: continue
        # order neighbours round the vertex through the faces
        nxt = {}
        ok = True
        for i in fs:
            f = [int(x) for x in F[i]]; j = f.index(v); a, b = f[(j + 1) % 3], f[(j + 2) % 3]
            if a in nxt: ok = False; break
            nxt[a] = b
        if not ok or set(nxt) != set(nxt.values()): continue
        ring = [next(iter(nxt))]
        while len(ring) < len(nxt):
            ring.append(nxt[ring[-1]])
        if nxt[ring[-1]] != ring[0]: continue
        signs = []
        for w in ring:
            k = ekey.get((min(v, w), max(v, w)))
            if k is None: signs = None; break
            d = c['e_dih'][k]
            if abs(d) >= thr: signs.append(1 if d > 0 else -1)
        if signs is None: continue
        n = sum(1 for i in range(len(signs)) if signs[i] != signs[i - 1]) if len(signs) > 1 else 0
        out[v] = n
    return out

def measure(c):
    cl.topology(c)
    F, C, V = c['F'], c['C'], c['V']; area = c['area']; LC = c['LC']; LN = c['LN']
    skin = c['drawn'] & (c['A'] == 0)
    ea, eb, el, dih = c['e_a'], c['e_b'], c['e_len'], c['e_dih']
    both = skin[ea] & skin[eb]
    ea, eb, el, dih, ev = ea[both], eb[both], el[both], dih[both], c['e_v'][both]
    used = np.unique(F[skin]); hgt = V[used][:, 1].max() - V[used][:, 1].min()
    out = dict(faces=int(skin.sum()), height=float(hgt))
    pl = (C & 127).astype(float)
    litf = LC.mean(1).astype(float)                       # lit lightness per face (0..127)
    lnf = LN[F].mean(1).astype(float)                     # light number per face
    sm = np.array([same_mat(int(C[a]), int(C[b])) for a, b in zip(ea, eb)])
    samecol = C[ea] == C[eb]
    adih = np.abs(dih)
    # ---------- dihedral distribution inside one material (edge-length weighted)
    w = el[sm]; d = adih[sm]; tot = w.sum()
    for lo, hi, name in ((0, 6, 'flat'), (6, 15, 'gentle'), (15, 30, 'mush'), (30, 50, 'firm'), (50, 181, 'hard')):
        out['dih_' + name] = float(w[(d >= lo) & (d < hi)].sum() / tot)
    out['dih_med'] = cl.cmesh.wquant(d, w, .5)
    g = (adih >= 6) & (adih < 30) & sm
    out['concave_of_soft'] = float(el[g & (dih < 0)].sum() / max(el[g].sum(), 1e-9))
    g2 = (adih >= 30) & sm
    out['concave_of_firm'] = float(el[g2 & (dih < 0)].sum() / max(el[g2].sum(), 1e-9))
    # ---------- crumple: sign changes of the fold round a vertex
    sc = ring_sign_changes(c)
    vs = [v for v in sc if skin[c['vf'][v]].all()]
    n4 = np.array([sc[v] for v in vs])
    out['saddle_vertices'] = float((n4 >= 4).mean()) if len(n4) else 0.0
    out['mixed_vertices'] = float((n4 >= 2).mean()) if len(n4) else 0.0
    # ---------- light number roughness at vertices (the vote result), in light-number units (128 = paint as painted)
    lap = []; ext = {3: 0, 6: 0, 10: 0}; nv = 0; lapw = []
    varea = collections.Counter()
    for i in np.nonzero(skin)[0]:
        for v in F[i]: varea[int(v)] += area[i] / 3
    for v in vs:
        nb = [w_ for w_ in c['vv'][v] if LN[w_] > -900]
        if len(nb) < 4 or LN[v] < -900: continue
        dn = LN[v] - LN[nb]; nv += 1
        lap.append(abs(float(dn.mean()))); lapw.append(varea[v])
        for t in ext:
            if (dn >= t).all() or (dn <= -t).all(): ext[t] += 1
    lap = np.array(lap); lapw = np.array(lapw)
    out['light_lap_med'] = float(np.median(lap)); out['light_lap_mean'] = float(lap.mean()); out['light_lap_p90'] = float(np.quantile(lap, .9))
    out['light_lap_areaw'] = float((lap * lapw).sum() / lapw.sum())
    for t in ext: out[f'light_extrema_{t}'] = ext[t] / max(nv, 1)
    # light range over the skin for scale
    lnu = LN[used]; lnu = lnu[lnu > -900]
    out['light_sd'] = float(lnu.std())
    out['light_lap_rel'] = out['light_lap_mean'] / max(out['light_sd'], 1e-9)
    # ---------- face-level roughness of LIT lightness against same-material neighbours: lit, paint only, light only
    nbs = collections.defaultdict(list)
    for a, b, s_ in zip(ea, eb, sm):
        if s_: nbs[int(a)].append(int(b)); nbs[int(b)].append(int(a))
    medln = float(np.median(lnf[skin]))
    r_lit, r_paint, r_light, wts = [], [], [], []
    isl = {2: 0, 4: 0, 6: 0}; isl_a = {2: 0.0, 4: 0.0, 6: 0.0}; n3 = 0; a3 = 0.0
    pisl = 0; pisl_a = 0.0
    for i in np.nonzero(skin)[0]:
        nb = nbs.get(int(i), [])
        if len(nb) < 2: continue
        nb = np.array(nb)
        r_lit.append(litf[i] - litf[nb].mean()); r_paint.append((pl[i] - pl[nb].mean()) * medln / 128); r_light.append(pl[i] * (lnf[i] - lnf[nb].mean()) / 128); wts.append(area[i])
        if len(nb) == 3:
            n3 += 1; a3 += area[i]; dd = litf[i] - litf[nb]
            for t in isl:
                if (dd >= t).all() or (dd <= -t).all(): isl[t] += 1; isl_a[t] += area[i]
            dp = pl[i] - pl[nb]
            if (dp > 0).all() or (dp < 0).all(): pisl += 1; pisl_a += area[i]
    r_lit, r_paint, r_light, wts = map(np.array, (r_lit, r_paint, r_light, wts))
    rms = lambda x: float(np.sqrt((x * x * wts).sum() / wts.sum()))
    out['rough_lit'] = rms(r_lit); out['rough_paint'] = rms(r_paint); out['rough_light'] = rms(r_light)
    out['rough_lit_med'] = float(np.median(np.abs(r_lit))); out['rough_paint_med'] = float(np.median(np.abs(r_paint))); out['rough_light_med'] = float(np.median(np.abs(r_light)))
    out['rough_lit_meanabs'] = float(np.abs(r_lit).mean())
    for t in isl: out[f'lit_island_{t}'] = isl[t] / max(n3, 1); out[f'lit_island_area_{t}'] = isl_a[t] / max(a3, 1e-9)
    out['paint_island'] = pisl / max(n3, 1); out['paint_island_area'] = pisl_a / max(a3, 1e-9)
    # ---------- paint steps within one material: where do they sit?
    step = sm & ~samecol
    out['tone_edge_share'] = float(el[step].sum() / max(el[sm].sum(), 1e-9))
    out['tone_edges_on_soft'] = float(el[step & (adih < 15)].sum() / max(el[step].sum(), 1e-9))     # tone step where the surface does not turn
    out['soft_edges_with_step'] = float(el[step & (adih < 15)].sum() / max(el[sm & (adih < 15)].sum(), 1e-9))
    dl = np.abs(pl[ea] - pl[eb])
    out['step_med'] = float(np.median(dl[step])) if step.any() else 0.0
    litjump = dl * (LN[ev].mean(1)) / 128
    out['litjump_soft_mean'] = float((litjump * el)[sm & (adih < 15)].sum() / max(el[sm & (adih < 15)].sum(), 1e-9))
    # tone boundary graph: how do boundaries run?
    deg = collections.defaultdict(list)
    for k in np.nonzero(step)[0]:
        a, b = ev[k]; deg[int(a)].append(k); deg[int(b)].append(k)
    turns = []; ends = junc = thru = 0
    for v, ks in deg.items():
        if len(ks) == 1: ends += 1
        elif len(ks) == 2:
            thru += 1
            o = [int(ev[k][0]) if int(ev[k][1]) == v else int(ev[k][1]) for k in ks]
            d1, d2 = V[v] - V[o[0]], V[o[1]] - V[v]
            cs = float(d1 @ d2 / max(np.linalg.norm(d1) * np.linalg.norm(d2), 1e-9)); turns.append(math.degrees(math.acos(max(-1, min(1, cs)))))
        else: junc += 1
    nvb = max(len(deg), 1)
    out['tb_turn_med'] = float(np.median(turns)) if turns else 0.0
    out['tb_sharp_turns'] = float(np.mean(np.array(turns) > 60)) if turns else 0.0
    out['tb_junctions'] = junc / nvb; out['tb_ends'] = ends / nvb
    # ---------- one-colour patches within one material: size distribution (the tone islands of the PAINT)
    parent = {int(i): int(i) for i in np.nonzero(skin)[0]}
    def find(x):
        while parent[x] != x: parent[x] = parent[parent[x]]; x = parent[x]
        return x
    for a, b, s_ in zip(ea, eb, samecol):
        if s_: parent[find(int(a))] = find(int(b))
    pf = collections.Counter(); pa = collections.Counter()
    for i in parent: r = find(i); pf[r] += 1; pa[r] += area[i]
    # a patch is a 'tone island' if every neighbour across its border is of the same material (it is tone, not an accent)
    pn = collections.defaultdict(list)
    for a, b, s_ in zip(ea, eb, sm):
        ra, rb = find(int(a)), find(int(b))
        if ra != rb: pn[ra].append(bool(s_)); pn[rb].append(bool(s_))
    tot_a = area[skin].sum()
    for kmax, name in ((1, '1'), (2, '2'), (4, '4')):
        ps = [r for r in pf if pf[r] <= kmax and pn[r] and all(pn[r])]
        out[f'tone_islands_{name}_per100'] = 100 * len(ps) / skin.sum(); out[f'tone_islands_{name}_area'] = float(sum(pa[r] for r in ps) / tot_a)
    out['patch_faces_med'] = float(np.median(list(pf.values()))); out['patches_per100'] = 100 * len(pf) / skin.sum()
    # ---------- triangle quality and evenness
    idx = np.nonzero(skin)[0]
    A_, B_, C_ = V[F[idx, 0]], V[F[idx, 1]], V[F[idx, 2]]
    e1, e2, e3 = np.linalg.norm(A_ - B_, axis=1), np.linalg.norm(B_ - C_, axis=1), np.linalg.norm(C_ - A_, axis=1)
    longest = np.maximum(e1, np.maximum(e2, e3)); asp = longest ** 2 / np.maximum(area[idx] * 2.3094, 1e-9)
    out['sliver_share'] = float((asp > 5).mean()); out['sliver_area'] = float(area[idx][asp > 5].sum() / tot_a); out['aspect_med'] = float(np.median(asp))
    ra = np.maximum(area[ea], area[eb]) / np.maximum(np.minimum(area[ea], area[eb]), 1e-9)
    out['nbr_area_ratio_med'] = float(np.median(ra)); out['nbr_area_ratio_over4'] = float((ra > 4).mean())
    out['area_cv'] = float(area[idx].std() / area[idx].mean())
    out['edge_med_rel'] = float(np.median(el) / hgt)
    out['tiny_face_share'] = float((np.sqrt(area[idx]) < 0.01 * hgt).mean())
    # ---------- welding
    Vi = np.round(V[used]).astype(int)
    pos = collections.Counter(map(tuple, Vi.tolist()))
    out['split_vertices'] = int(sum(n for n in pos.values() if n > 1))
    grid = collections.defaultdict(list)
    for j, p in zip(used, Vi): grid[tuple(p // 3)].append(int(j))
    near = 0
    vvs = c['vv']
    for j, p in zip(used, Vi):
        hit = False
        cx = tuple(p // 3)
        for dx in (-1, 0, 1):
            for dy in (-1, 0, 1):
                for dz in (-1, 0, 1):
                    for o in grid.get((cx[0] + dx, cx[1] + dy, cx[2] + dz), ()):
                        if o != j and o not in vvs[int(j)] and np.abs(V[o] - V[j]).max() <= 2: hit = True
        near += hit
    out['near_unwelded_vertices'] = near / len(used)
    out['open_edge_share'] = len(c['open']) / max(len(c['e_a']) + len(c['open']), 1)
    shells = collections.Counter(c['shell'][int(i)] for i in idx)
    out['shells'] = len(shells); out['largest_shell'] = max(shells.values()) / len(idx)
    out['small_shell_faces'] = float(sum(n for n in shells.values() if n <= 40) / len(idx))
    out['votes_med'] = float(np.median(c['votes'][used]))
    return out

def image_measures(c, px=420):
    """Blotch energy of the lit picture inside one material, away from material borders and the outline."""
    import cv2
    out = {}
    skin = c['drawn'] & (c['A'] == 0)
    used = np.unique(c['F'][skin]); hgt = np.ptp(c['V'][used][:, 1])
    sk = paintlib.skin(c['entry'], c['mesh']); mats, mat_of = paintlib.materials(sk)
    fm = np.full(len(c['F']), -1); fm[sk['idx']] = mat_of
    vals = []; vals2 = []; npx = 0
    for yaw in (25, 205):
        img, ids = cl.view(c, yaw, 8, ppu=px / hgt, faces=np.nonzero(skin)[0])
        g = cl.grey(img); m = fm[ids] ; m[ids < 0] = -1
        # interior: all pixels within 4 px have the same material
        mx = cv2.dilate(m.astype(np.float32), np.ones((9, 9))); mn = cv2.erode(m.astype(np.float32), np.ones((9, 9)))
        inner = (mx == mn) & (m >= 0)
        if inner.sum() < 50: continue
        wgt = inner.astype(np.float32)
        def blur(x, s): return cv2.GaussianBlur(x, (0, 0), s)
        for s, store in ((6.0, vals), (2.5, vals2)):
            mean = blur((g * wgt).astype(np.float32), s) / np.maximum(blur(wgt, s), 1e-3)
            hp = (g - mean)[inner]; store.append(hp)
        npx += int(inner.sum())
    if not vals: return dict(img_blotch=0.0)
    hp = np.concatenate(vals); hp2 = np.concatenate(vals2)
    out['img_blotch_rms'] = float(np.sqrt((hp ** 2).mean())); out['img_blotch_mad'] = float(np.abs(hp).mean())
    out['img_fine_rms'] = float(np.sqrt((hp2 ** 2).mean()))
    out['img_blotch_over4'] = float((np.abs(hp) > 4).mean())
    return out

def all_entries():
    return [(str(i), f'npc:{i}', n, st) for i, (n, st) in REAL.items()] + [(k, 'def:' + str(p), k, 'o') for k, p in OURS.items()]

def sep(real, ours):
    """Does it separate: (direction, how many of ours lie outside the full real range)"""
    lo, hi = min(real), max(real)
    return sum(1 for o in ours if o < lo or o > hi)

if __name__ == '__main__':
    store = HERE / 'data' / 'explore.json'; store.parent.mkdir(exist_ok=True)
    rows = json.loads(store.read_text()) if store.exists() and '--refresh' not in sys.argv else {}
    for key, entry, name, st in all_entries():
        if key in rows: continue
        c = cl.load(entry); r = measure(c); r.update(image_measures(c)); r['name'] = name; r['style'] = st; rows[key] = r
        print('measured', name, file=sys.stderr)
        store.write_text(json.dumps(rows, indent=0))
    keys = [k for k in next(iter(rows.values())) if k not in ('name', 'style')]
    real = [r for r in rows.values() if r['style'] != 'o']
    oursk = ['watcher-2', 'watcher-repaint', 'star-hound-3', 'warden-general-3']
    print(f"{'measure':28s} {'real min':>8s} {'p10':>7s} {'med':>7s} {'p90':>7s} {'max':>8s} | {'smooth':>7s} {'detail':>7s} | {'Mino':>7s} {'Tekton':>7s} | {'w2':>7s} {'w2-rep':>7s} {'hound3':>7s} {'WG3':>7s} | out")
    for k in keys:
        v = np.array([r[k] for r in real], float)
        s_ = np.median([r[k] for r in real if r['style'] == 's']); d_ = np.median([r[k] for r in real if r['style'] == 'd'])
        o = [rows[q][k] for q in oursk]
        flags = ''.join('<' if x < v.min() else '>' if x > v.max() else ('(' if x < np.quantile(v, .1) else ')' if x > np.quantile(v, .9) else '.') for x in o)
        print(f"{k:28s} {v.min():8.3f} {np.quantile(v, .1):7.3f} {np.median(v):7.3f} {np.quantile(v, .9):7.3f} {v.max():8.3f} | {s_:7.3f} {d_:7.3f} | {rows['12812'][k]:7.3f} {rows['7541'][k]:7.3f} | " + ' '.join(f'{x:7.3f}' for x in o) + ' | ' + flags)
