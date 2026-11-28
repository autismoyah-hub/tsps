"""Creature mesh measurements (research-creatures). Everything here reads a REST mesh as the client assembles it:
`asset_npc.exe CACHE merged ENTRY out.txt` (npc_model::mesh: every model of the definition combined, recoloured).

    mesh = load('npc:106')  or  load('def:PATH.json')  or  load_file(PATH.txt)
    m = measure(mesh)       -> dict of the measures described in CREATURES.md (one flat dict of numbers)

Coordinates are the cache's: x left-right, y DOWN, z along the body. Colours are 16-bit HSL after recolour.
A face is DRAWN unless its alpha is 255 or its render type is 2 or more (those are kept only for lighting votes).
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, pathlib, subprocess, json, math, collections, hashlib
import numpy as np
import cv2
HERE = pathlib.Path(__file__).parent
sys.path.insert(0, str(HERE)); sys.path.insert(1, _AP.fix(r'<ROOT>\studies\research-npc'))
EXE = _AP.fix(r'<CLIENT_BIN>\asset_npc.exe')
CACHE = _AP.fix(r'<OSRS_CACHE_239>')
DATA = HERE / 'data'
LIGHT = np.array([-30.0, -50.0, -30.0]); LIGHT /= np.linalg.norm(LIGHT)

def hsl(c): return (c >> 10) & 63, (c >> 7) & 7, c & 127

def entry_to_exe(entry, workdir):
    """npc:ID stays; def:X.json becomes def:X.npcdef through the NPC harness's own converter."""
    if entry.startswith('def:') and entry.lower().endswith('.json'):
        import npc
        workdir.mkdir(parents=True, exist_ok=True)
        return 'def:' + str(npc.to_npcdef(pathlib.Path(entry[4:]), workdir))
    return entry

def load_file(path):
    V, L, F, C, P, T, D, A, FL = [], [], [], [], [], [], [], [], []
    head = ''
    for line in pathlib.Path(path).read_text().splitlines():
        p = line.split()
        if not p: continue
        if p[0] == '#': head = line
        elif p[0] == 'v': V.append((int(p[1]), int(p[2]), int(p[3]))); L.append(int(p[4]))
        elif p[0] == 'f':
            F.append((int(p[1]), int(p[2]), int(p[3]))); C.append(int(p[4])); P.append(int(p[5])); T.append(int(p[6]))
            D.append(int(p[7])); A.append(int(p[8])); FL.append(int(p[9]))
    return dict(V=np.array(V, float).reshape(-1, 3), L=np.array(L, int), F=np.array(F, int).reshape(-1, 3), C=np.array(C, int), P=np.array(P, int),
                T=np.array(T, int), D=np.array(D, int), A=np.array(A, int), FL=np.array(FL, int), head=head)

def load(entry, refresh=False):
    """The assembled rest mesh of an entry, cached under data/merged/."""
    d = DATA / 'merged'; d.mkdir(parents=True, exist_ok=True)
    name = entry.replace(':', '-') if entry.startswith('npc:') else 'def-' + pathlib.Path(entry[4:]).stem + '-' + hashlib.md5(entry.encode()).hexdigest()[:6]
    out = d / (name + '.txt')
    if refresh or not out.exists() or not entry.startswith('npc:'):
        r = subprocess.run([EXE, CACHE, 'merged', entry_to_exe(entry, d / 'defs'), str(out)], capture_output=True, text=True)
        if r.returncode: raise SystemExit(r.stdout + r.stderr)
    return load_file(out)

def face_geometry(m):
    V, F = m['V'], m['F']
    a, b, c = V[F[:, 0]], V[F[:, 1]], V[F[:, 2]]
    n = np.cross(b - a, c - a); area = np.linalg.norm(n, axis=1) / 2
    unit = n / np.maximum(np.linalg.norm(n, axis=1), 1e-9)[:, None]
    cen = (a + b + c) / 3
    return area, unit, cen

def outward_sign(m, area, unit, cen):
    """+1 if the winding normals point out of the body, -1 if in (decided over the whole mesh)."""
    centre = (cen * area[:, None]).sum(0) / max(area.sum(), 1e-9)
    return 1.0 if float(((cen - centre) * unit).sum(1) @ area) >= 0 else -1.0

def wquant(x, w, q):
    o = np.argsort(x); x, w = np.asarray(x)[o], np.asarray(w)[o]; c = np.cumsum(w)
    return float(x[min(np.searchsorted(c, q * c[-1]), len(x) - 1)])

def wcorr(x, y, w):
    if len(x) < 3 or w.sum() <= 0: return None
    mx, my = np.average(x, weights=w), np.average(y, weights=w)
    vx, vy = np.average((x - mx) ** 2, weights=w), np.average((y - my) ** 2, weights=w)
    if vx < 1e-9 or vy < 1e-9: return None
    return float(np.average((x - mx) * (y - my), weights=w) / math.sqrt(vx * vy))

HMAP = {}
def hue_clusters(colours):
    """Neighbouring hues (one step apart, wrapping at 64) are one family: real palettes shift hue a step with tone."""
    hs = sorted({hsl(int(c))[0] for c in colours if hsl(int(c))[1] > 0 and 6 < hsl(int(c))[2] < 122})
    m = {}
    for i, h in enumerate(hs): m[h] = m[hs[i - 1]] if i and h - hs[i - 1] <= 1 else h
    if len(hs) > 1 and hs[0] + 64 - hs[-1] <= 1:
        last = m[hs[-1]]
        for h in hs:
            if m[h] == last: m[h] = m[hs[0]]
    return m
def family(c):
    """Hue family of a colour: 'k' near black, 'w' near white, 'g' unsaturated, else its hue cluster (HMAP)."""
    h, s, l = hsl(int(c))
    if l <= 6: return 'k'
    if l >= 122: return 'w'
    if s == 0: return 'g'
    return HMAP.get(h, h)

def silhouettes(V, F, px_per_unit=None, size=256):
    """Binary outline pictures: side (z across, height up), front (x across), top (x across, z down)."""
    lo, hi = V.min(0), V.max(0); ext = np.maximum(hi - lo, 1)
    s = px_per_unit if px_per_unit else (size - 8) / ext.max()
    out = {}
    for name, (ax, ay) in dict(side=(2, 1), front=(0, 1), top=(0, 2)).items():
        w, h = int(ext[ax] * s) + 9, int(ext[ay] * s) + 9
        im = np.zeros((h, w), np.uint8)
        pts = np.stack([(V[:, ax] - lo[ax]) * s + 4, (V[:, ay] - lo[ay]) * s + 4], 1)
        tri = np.round(pts[F] * 16).astype(np.int32)
        for t in tri: cv2.fillConvexPoly(im, t, 255, lineType=cv2.LINE_8, shift=4)
        out[name] = im
    return out

def outline_numbers(im):
    cs, _ = cv2.findContours(im, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_NONE)
    if not cs: return None
    per = sum(cv2.arcLength(c, True) for c in cs); area = float((im > 0).sum())
    hull = cv2.convexHull(np.concatenate(cs)); hper = cv2.arcLength(hull, True); harea = max(cv2.contourArea(hull), 1.0)
    return dict(ragged=per / max(hper, 1e-9), solidity=min(area / harea, 1.0))

def thickness(V, F, unit, cen):
    """For each face, the distance from its centre straight inward to the next face (nan if none)."""
    a, b, c = V[F[:, 0]], V[F[:, 1]], V[F[:, 2]]
    e1, e2 = b - a, c - a
    out = np.full(len(F), np.nan)
    for i in range(len(F)):
        d = -unit[i]; o = cen[i] + d * 0.05
        p = np.cross(d, e2); det = (e1 * p).sum(1)
        good = np.abs(det) > 1e-9
        inv = np.where(good, 1.0 / np.where(good, det, 1), 0)
        t0 = o - a; u = (t0 * p).sum(1) * inv
        q = np.cross(t0, e1); v = (q @ d) * inv; t = (e2 * q).sum(1) * inv
        hit = good & (u >= -1e-6) & (v >= -1e-6) & (u + v <= 1 + 1e-6) & (t > 0.05)
        hit[i] = False
        if hit.any(): out[i] = t[hit].min()
    return out

def measure(m, scale=(128, 128), game_px_per_unit=0.445):
    V, F, C, L = m['V'], m['F'], m['C'], m['L']
    out = {}
    drawn = (m['A'] != 255) & (m['T'] < 2)
    out['faces_all'] = int(len(F)); out['hidden_vote_faces'] = int((~drawn).sum())
    area_all, unit_all, cen_all = face_geometry(m)
    sign = outward_sign(m, area_all, unit_all, cen_all)
    keep = drawn & (area_all > 0)
    Fd, Cd = F[keep], C[keep]; area, unit, cen = area_all[keep], unit_all[keep] * sign, cen_all[keep]
    nf = len(Fd); tot = float(area.sum()) or 1.0
    used = np.unique(Fd)
    out['faces'] = int(nf); out['vertices'] = int(len(V)); out['zero_area_faces'] = int((drawn & (area_all == 0)).sum())
    sc = np.array([scale[0], scale[1], scale[0]], float) / 128
    Vs = V * sc; lo, hi = Vs[used].min(0), Vs[used].max(0); ext = hi - lo
    out['size_x'], out['size_y'], out['size_z'] = (float(round(e)) for e in ext)
    out['surface'] = tot * float(sc[0] * sc[1])          # rough, for a faces-per-surface figure only
    out['faces_per_1000sq'] = 1000 * nf / max(out['surface'], 1)
    # ---- face size relative to the body
    rel = area / tot
    out['face_med_share_e4'] = float(np.median(rel) * 1e4)
    out['face_p90_p10'] = float(np.quantile(area, .9) / max(np.quantile(area, .1), 1e-9))
    o = np.argsort(-area); cum = np.cumsum(area[o]) / tot
    out['faces_for_half_surface'] = float((np.searchsorted(cum, .5) + 1) / nf)      # small = a few big faces carry the body
    out['largest_face_share'] = float(rel.max())
    out['edge_med_over_height'] = float(np.median(np.sqrt(area * 2)) / max(V[used][:, 1].max() - V[used][:, 1].min(), 1))
    # ---- colour
    by = collections.Counter()
    for c, a in zip(Cd, area): by[int(c)] += a
    out['colours'] = len(by)
    acc = 0; n90 = 0
    for c, a in by.most_common():
        acc += a; n90 += 1
        if acc >= .9 * tot: break
    out['colours_for_90'] = n90
    ls = np.array([hsl(int(c))[2] for c in Cd]); ss = np.array([hsl(int(c))[1] for c in Cd])
    out['light_med'] = wquant(ls, area, .5); out['light_p10'] = wquant(ls, area, .1); out['light_p90'] = wquant(ls, area, .9)
    out['sat_med'] = wquant(ss, area, .5)
    HMAP.clear(); HMAP.update(hue_clusters([c for c, a in by.items() if a >= .004 * tot]))
    fam = collections.defaultdict(lambda: collections.Counter())
    for c, a in by.items(): fam[family(c)][c] += a
    fams = {k: v for k, v in fam.items() if sum(v.values()) >= .03 * tot}
    out['hue_families'] = len(fams)
    tones = {k: len({hsl(c)[2] for c, a in v.items() if a >= .004 * tot}) for k, v in fams.items()}
    w = np.array([sum(fams[k].values()) for k in fams]); t = np.array([tones[k] for k in fams])
    out['tones_per_hue'] = float((w * t).sum() / max(w.sum(), 1e-9)) if len(fams) else 0.0
    out['tones_main_hue'] = int(t[np.argmax(w)]) if len(fams) else 0
    main = max(fams, key=lambda k: sum(fams[k].values())) if fams else None
    out['main_hue_share'] = float(sum(fams[main].values()) / tot) if fams else 0.0
    mls = sorted({hsl(c)[2] for c, a in fams[main].items() if a >= .004 * tot}) if fams else []
    out['main_hue_light_span'] = float(mls[-1] - mls[0]) if mls else 0.0
    out['main_hue_light_step'] = float(np.median(np.diff(mls))) if len(mls) > 1 else 0.0
    out['accent_share'] = float(sum(a for k, v in fam.items() if sum(v.values()) < .08 * tot for a in v.values()) / tot)
    # tone by facing: within each hue family with two or more tones, does lightness follow the normal?
    famf = np.array([str(family(c)) for c in Cd]); up = -unit[:, 1]; lit = unit @ LIGHT
    cu, cl, cw = [], [], []
    for k in fams:
        if tones[k] < 2: continue
        s = famf == str(k); r1, r2 = wcorr(ls[s].astype(float), up[s], area[s]), wcorr(ls[s].astype(float), lit[s], area[s])
        if r1 is not None: cu.append(r1); cl.append(r2 or 0.0); cw.append(area[s].sum())
    out['tone_follows_up'] = float(np.average(cu, weights=cw)) if cu else 0.0
    out['tone_follows_light'] = float(np.average(cl, weights=cw)) if cu else 0.0
    # tone by part: lightness variance inside a hue family explained by the face's label (adjusted eta squared)
    lab_face = L[Fd[:, 0]]
    if (L != 0).any():
        num = den = 0.0
        for k in fams:
            if tones[k] < 2: continue
            s = famf == str(k); x = ls[s].astype(float); w = area[s]; g = lab_face[s]
            mu = np.average(x, weights=w); sst = float((w * (x - mu) ** 2).sum()); ssw = 0.0
            for q in np.unique(g):
                t = g == q; ssw += float((w[t] * (x[t] - np.average(x[t], weights=w[t])) ** 2).sum())
            n, kk = int(s.sum()), len(np.unique(g))
            if sst > 0 and n > kk + 1: num += w.sum() * max(0.0, 1 - (ssw / sst) * (n - 1) / (n - kk)); den += w.sum()
        out['tone_by_part'] = num / den if den else 0.0
    # tone by height: is the creature lighter above and darker below, whatever the facing?
    ch = []
    for k in fams:
        if tones[k] < 2: continue
        s = famf == str(k); r1 = wcorr(ls[s].astype(float), -cen[s][:, 1], area[s])
        if r1 is not None: ch.append((r1, area[s].sum()))
    out['tone_follows_height'] = float(np.average([c for c, _ in ch], weights=[w for _, w in ch])) if ch else 0.0
    # face shape and regularity of the split
    ea = np.linalg.norm(V[Fd[:, 0]] - V[Fd[:, 1]], axis=1); eb = np.linalg.norm(V[Fd[:, 1]] - V[Fd[:, 2]], axis=1); ec = np.linalg.norm(V[Fd[:, 2]] - V[Fd[:, 0]], axis=1)
    longest = np.maximum(ea, np.maximum(eb, ec)); aspect = longest ** 2 / np.maximum(area * 2.3094, 1e-9)     # 1 for an equilateral triangle
    out['sliver_face_share'] = float((aspect > 5).mean()); out['aspect_med'] = float(np.median(aspect))
    val = np.bincount(Fd.ravel(), minlength=len(V))[used]
    out['valence_sd'] = float(val.std()); out['valence_6_share'] = float((val == 6).mean()); out['valence_med'] = float(np.median(val))
    key = set(map(tuple, V[used].astype(int).tolist()))
    out['mirror_vertex_share'] = float(np.mean([(-x, y, z) in key for x, y, z in key]))
    # ---- edges: folds and colour boundaries
    edges = collections.defaultdict(list)
    for i, f in enumerate(Fd):
        for a, b in ((f[0], f[1]), (f[1], f[2]), (f[2], f[0])): edges[(min(a, b), max(a, b))].append(i)
    Vn = V
    e2 = [(k, v) for k, v in edges.items() if len(v) == 2]
    if e2:
        ia = np.array([v[0] for _, v in e2]); ib = np.array([v[1] for _, v in e2])
        el = np.linalg.norm(Vn[[k[0] for k, _ in e2]] - Vn[[k[1] for k, _ in e2]], axis=1)
        fold = np.degrees(np.arccos(np.clip((unit[ia] * unit[ib]).sum(1), -1, 1)))
        change = Cd[ia] != Cd[ib]; flat_e = fold < 20
        same_fam = famf[ia] == famf[ib]
        out['fold_med_deg'] = wquant(fold, el, .5)
        out['fold_over30_share'] = float(el[fold > 30].sum() / el.sum())
        out['fold_over60_share'] = float(el[fold > 60].sum() / el.sum())
        out['fold_under5_share'] = float(el[fold < 5].sum() / el.sum())
        out['colour_edge_share'] = float(el[change].sum() / el.sum())                         # edge length that is a colour boundary
        out['insurface_colour_edge_share'] = float(el[change & flat_e].sum() / max(el[flat_e].sum(), 1e-9))   # painted in a surface
        out['colour_edges_in_surface'] = float(el[change & flat_e].sum() / max(el[change].sum(), 1e-9))
        out['tone_step_edges_same_hue'] = float(el[change & same_fam].sum() / max(el[change].sum(), 1e-9))
        dl = np.abs(ls[ia] - ls[ib])[change & same_fam]
        out['tone_step_med'] = float(np.median(dl)) if len(dl) else 0.0
    else:
        for k in ('fold_med_deg', 'fold_over30_share', 'fold_over60_share', 'colour_edge_share', 'insurface_colour_edge_share', 'colour_edges_in_surface', 'tone_step_edges_same_hue', 'tone_step_med'): out[k] = 0.0
    ne = len(edges)
    out['open_edge_share'] = sum(1 for v in edges.values() if len(v) == 1) / max(ne, 1)
    out['nonmanifold_edge_share'] = sum(1 for v in edges.values() if len(v) > 2) / max(ne, 1)
    # colour regions: connected patches of one colour
    parent = list(range(nf))
    def find(i):
        while parent[i] != i: parent[i] = parent[parent[i]]; i = parent[i]
        return i
    for k, v in edges.items():
        for j in v[1:]:
            if Cd[v[0]] == Cd[j]: parent[find(j)] = find(v[0])
    roots = np.array([find(i) for i in range(nf)])
    reg_area = collections.Counter(); reg_faces = collections.Counter()
    for r, a in zip(roots, area): reg_area[r] += a; reg_faces[r] += 1
    out['colour_regions'] = len(reg_area); out['regions_per_100_faces'] = 100 * len(reg_area) / nf
    out['faces_per_region_med'] = float(np.median(list(reg_faces.values())))
    out['largest_region_share'] = max(reg_area.values()) / tot
    # small contrasting islands = painted or modelled detail (eyes, teeth, claws, markings, straps)
    neigh = collections.defaultdict(set)
    for k, v in edges.items():
        for i in v:
            for j in v:
                if roots[i] != roots[j]: neigh[roots[i]].add(roots[j])
    reg_col = {r: int(Cd[r]) for r in reg_area}
    # shells (connected by shared vertices)
    vp = {int(v): int(v) for v in used}
    def vfind(i):
        while vp[i] != i: vp[i] = vp[vp[i]]; i = vp[i]
        return i
    for f in Fd:
        a, b, c = (vfind(int(x)) for x in f); vp[b] = a; vp[vfind(int(f[2]))] = a
    shell = np.array([vfind(int(f[0])) for f in Fd]); shell_faces = collections.Counter(shell.tolist())
    out['shells'] = len(shell_faces); out['shells_8plus'] = sum(1 for n in shell_faces.values() if n >= 8)
    out['largest_shell_face_share'] = max(shell_faces.values()) / nf
    islands = dict(inlaid=0, own_shell=0, offset=0); isl_area = 0.0
    Dd = m['D'][keep]
    for r, a in reg_area.items():
        if a > .012 * tot: continue
        h, s, l = hsl(reg_col[r]); ns = [hsl(reg_col[q]) for q in neigh[r]]
        if ns and not all(abs(l - q[2]) >= 12 or (family(reg_col[r]) != family(reg_col[x]))for q, x in zip(ns, neigh[r])): continue
        idx = np.nonzero(roots == r)[0]
        if (Dd[idx] != 0).any(): islands['offset'] += 1
        elif not ns or shell_faces[int(shell[idx[0]])] <= max(reg_faces[r] * 3, 24): islands['own_shell'] += 1
        else: islands['inlaid'] += 1
        isl_area += a
    out['detail_islands'] = sum(islands.values()); out['detail_inlaid'] = islands['inlaid']; out['detail_own_shell'] = islands['own_shell']
    out['detail_offset'] = islands['offset']; out['detail_area_share'] = isl_area / tot
    out['detail_islands_per_100_faces'] = 100 * out['detail_islands'] / nf
    # ---- shading flags and layers
    out['flat_face_share'] = float((m['T'][keep] & 1).mean()); out['flat_area_share'] = float(area[(m['T'][keep] & 1) == 1].sum() / tot)
    out['depth_offset_face_share'] = float((Dd != 0).mean()); out['alpha_face_share'] = float(((m['A'][keep] > 0)).mean())
    out['priority_levels'] = int(len(set(m['P'][keep].tolist())))
    # ---- welding
    pos = collections.Counter(map(tuple, V[used].astype(int).tolist()))
    out['split_vertex_share'] = sum(n for n in pos.values() if n > 1) / max(len(used), 1)
    # ---- labels
    labs = set(L[used].tolist()); out['labels'] = len(labs) if (L != 0).any() else 0
    fl = L[Fd]; bridge = (fl[:, 0] != fl[:, 1]) | (fl[:, 1] != fl[:, 2])
    out['bridge_face_share'] = float(bridge.mean()) if out['labels'] else 0.0
    out['vertices_per_label'] = len(used) / max(out['labels'], 1) if out['labels'] else 0.0
    # ---- where the faces go: core against extremities (elliptical radius from the middle of the bounds)
    lo0, hi0 = V[used].min(0), V[used].max(0); ctr = (lo0 + hi0) / 2; half = np.maximum((hi0 - lo0) / 2, 1)
    r = np.sqrt((((cen - ctr) / half) ** 2).sum(1))
    extm = r >= .62
    fa, aa = float(extm.mean()), float(area[extm].sum() / tot)
    out['extremity_face_share'] = fa; out['extremity_area_share'] = aa
    out['extremity_density_ratio'] = (fa / max(aa, 1e-9)) / max((1 - fa) / max(1 - aa, 1e-9), 1e-9)
    # ---- where the faces go by how THICK the body is under each face (a ray from the face inward to the far wall):
    # trunk = thick, limbs = middling (head, neck, thighs), thin = shins, feet, hands, horns, tails, fingers
    thick = thickness(V, Fd, unit, cen)
    ok = np.isfinite(thick)
    if ok.sum() > 10:
        tmax = wquant(thick[ok], area[ok], .97)
        cls = np.where(~ok, 0, np.where(thick >= .5 * tmax, 2, np.where(thick >= .2 * tmax, 1, 0)))
        for k, name in ((2, 'trunk'), (1, 'limb'), (0, 'thin')):
            out[f'{name}_face_share'] = float((cls == k).mean()); out[f'{name}_area_share'] = float(area[cls == k].sum() / tot)
        dens = lambda k: (cls == k).mean() / max(area[cls == k].sum() / tot, 1e-9)
        out['thin_to_trunk_density'] = float(dens(0) / max(dens(2), 1e-9))
        big = area >= np.quantile(area, .9)
        out['big_faces_on_trunk'] = float((cls[big] == 2).mean())
    hgt = (hi0[1] - cen[:, 1]) / max(hi0[1] - lo0[1], 1)      # 0 at the ground, 1 at the top
    out['feet_zone_face_share'] = float((hgt < .12).mean()); out['feet_zone_area_share'] = float(area[hgt < .12].sum() / tot)
    # ---- outline
    for tag, s in (('', None), ('_game', game_px_per_unit)):
        rs, so = [], []
        Vu = Vs if s else V
        for name, im in silhouettes(Vu, Fd, s).items():
            n = outline_numbers(im)
            if n: rs.append(n['ragged']); so.append(n['solidity']); out[f'ragged_{name}{tag}'] = n['ragged']; out[f'solidity_{name}{tag}'] = n['solidity']
        out['ragged' + tag] = float(np.mean(rs)); out['solidity' + tag] = float(np.mean(so))
    return out

if __name__ == '__main__':
    for e in sys.argv[1:]:
        r = measure(load(e)); print(e); print(json.dumps({k: (round(v, 4) if isinstance(v, float) else v) for k, v in r.items()}, indent=0))
