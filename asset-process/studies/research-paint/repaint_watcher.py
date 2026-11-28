"""Step 4: repaint the second stone watcher to its paint plan WITHOUT touching its shape.

usage: repaint_watcher.py [--out NAME] [--strips N] [--seed TEXT]
reads  builds/creature-stone-watcher-2/creature-stone-watcher-2.bin and .json (never written to)
       plans/watcher-2-plan.json (the label lists of its parts are filled in on the first run)
writes repaint/NAME.bin, repaint/NAME.json, repaint/NAME-log.json

Vertices, labels, face order of the original faces, alpha, render types and depth are kept. Only face COLOURS change,
except with --strips N: up to N triangles on plate borders are re-split (new vertices ON existing edges, so the
surface and outline do not move; each new vertex takes the label of the nearer end of its edge) to give a light
edge line a face boundary of its own.

Method (every step follows a line of the plan):
 1. each face's MATERIAL is read from its old colour (old brass -> bronze, old leather -> leather, every stone-like
    colour including the old limestone and grey -> the ONE stone ramp);
 2. deliberate dark stone (old lightness 16 or less: sockets, nostrils, mouth, joints) stays as the ramp's two
    darkest tones; deliberate pale stone marks of two or more faces stay as the part's palest allowed tones;
    one-triangle marks (splinters) are dropped into the plane they lie in;
 3. the rest of the stone is toned BY PLANE: connected faces within 16 degrees of each other (28 of the first) on
    one part of one shell are one plane, one tone; the tone is the part's base tone (plan), one step up on a raised
    plane (borders mostly convex) or a plate, one down in a recess, and a fixed step from a table keyed on the
    plane's place with |x|, so left and right planes agree; never by facing;
 4. planes smaller than two faces take the tone of their largest neighbour.
"""
import sys, pathlib, json, hashlib, collections, math
import numpy as np
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import paintlib as pl, paintplan
import meshio
B = HERE.parent
SRC = B / 'creature-stone-watcher-2'
PLAN = HERE / 'plans' / 'watcher-2-plan.json'

def as_cmesh(m):
    nf = len(m['faces'])
    return dict(V=np.array(m['vertices'], float), L=np.array(m['vertex_skins'], int), F=np.array(m['faces'], int), C=np.array(m['colors'], int),
                P=np.array(m['priorities'], int), T=np.array(m['render_types'], int), D=np.array(m['depth'], int), A=np.array(m['alpha'], int), FL=np.array(m['face_skins'], int), head='')

def cmesh_q(x, w):
    import cmesh
    return cmesh.wquant(np.asarray(x, float), np.asarray(w, float), .5)

def nearest(ramp, l): return int(np.argmin([abs(c[2] - l) for c in ramp]))

def split_edge(m, a, b, t):
    """Insert a vertex on edge (a, b) at a + t (b - a) and split every face using that edge in two. Returns its index."""
    V = m['vertices']; pa, pb = np.array(V[a], float), np.array(V[b], float)
    p = tuple(int(round(x)) for x in pa + t * (pb - pa))
    if p == tuple(V[a]) or p == tuple(V[b]): return None
    n = len(V); V.append(p); m['vertex_skins'].append(m['vertex_skins'][a] if t < 0.5 else m['vertex_skins'][b])
    for i in range(len(m['faces'])):
        f = m['faces'][i]
        if a in f and b in f:
            c = [v for v in f if v != a and v != b][0]
            ia = f.index(a); order = (f[ia], f[(ia + 1) % 3], f[(ia + 2) % 3])           # a, then the winding
            if order[1] == b: f1, f2 = (a, n, c), (n, b, c)
            else: f1, f2 = (a, c, n), (n, c, b)
            m['faces'][i] = f1; m['faces'].append(f2)
            for k in ('colors', 'priorities', 'render_types', 'alpha', 'depth', 'face_skins'): m[k].append(m[k][i])
    return n

def run(out='watcher-2-repaint', strips=0, seed='meridian'):
    m = meshio.load(SRC / 'creature-stone-watcher-2.bin'); base_def = json.loads((SRC / 'creature-stone-watcher-2.json').read_text())
    plan = json.loads(PLAN.read_text())
    sk = pl.skin('watcher', mesh=as_cmesh(m))
    if not plan['parts']['head']['labels']:
        for k, v in paintplan.humanoid_labels(sk).items(): plan['parts'][k]['labels'] = v
        PLAN.write_text(json.dumps(plan, indent=1))
    part, names = paintplan.parts_from_plan(sk, plan)
    S, B_, Lth = plan['materials']['stone']['ramp'], plan['materials']['bronze']['ramp'], plan['materials']['leather']['ramp']
    F, C, area, unit, cen, tot = sk['F'], sk['C'], sk['area'], sk['unit'], sk['cen'], sk['tot']
    nf = len(F); idx = sk['idx']                                   # idx: skin face -> face of the file
    import inspect_shells
    shell, nshell = inspect_shells.shells(sk)
    body = int(np.argmax(np.bincount(shell)))
    ea, eb, el, fold, convex = sk['e_a'], sk['e_b'], sk['e_len'], sk['e_fold'], sk['e_convex']
    nb = collections.defaultdict(list)
    for k, (a, b) in enumerate(zip(ea, eb)): nb[a].append((b, k)); nb[b].append((a, k))
    # ---- 1. material of every face from its old colour
    hs = np.array([pl.hsl(c) for c in C])
    mat = np.where((hs[:, 0] >= 7) & (hs[:, 0] <= 9) & (hs[:, 1] >= 4), 1, np.where((hs[:, 0] >= 5) & (hs[:, 0] <= 7) & (hs[:, 1] == 2), 2, 0))
    old_patch = sk['patch']; pn = np.bincount(old_patch); pa = np.bincount(old_patch, weights=area)
    # old neighbours' mean lightness per patch
    pl_sum = collections.Counter(); pl_w = collections.Counter()
    for a, b, l in zip(ea, eb, el):
        if old_patch[a] != old_patch[b]:
            pl_sum[old_patch[a]] += hs[b, 2] * l; pl_w[old_patch[a]] += l; pl_sum[old_patch[b]] += hs[a, 2] * l; pl_w[old_patch[b]] += l
    nbl = lambda p: pl_sum[p] / pl_w[p] if pl_w[p] else None
    log = collections.Counter()
    new = np.full(nf, -1, int); fixed = np.zeros(nf, bool)
    pp = plan['paint']
    def rng(i, name): return pp[names[part[i]]]['ramps'].get(name) if names[part[i]] in pp else None
    for i in range(nf):
        p = old_patch[i]
        if mat[i] == 1:
            if pn[p] == 1 and pa[p] < 0.0006 * tot and hs[i, 2] < 80: mat[i] = 0; log['bronze splinters dropped'] += 1; continue
            r = rng(i, 'bronze') or [0, 6]; t = min(max(nearest(B_, hs[i, 2]), r[0]), r[1]); new[i] = pl.pack(*B_[t]); fixed[i] = True
        elif mat[i] == 2:
            if rng(i, 'leather') is None: mat[i] = 0; log['leather off its part -> stone'] += 1; continue
            new[i] = pl.pack(*Lth[nearest(Lth, hs[i, 2])]); fixed[i] = True
    # the horns are bronze-bound: their outer ends are bronze (the plan's second material must be a mass somewhere)
    ground = sk['V'][np.unique(F)][:, 1].max(); horn = np.zeros(nf, bool)
    for i in range(nf):
        if not fixed[i] and mat[i] == 0 and shell[i] == body and names[part[i]] == 'head' and ground - cen[i, 1] > 448 and abs(cen[i, 0]) > 62: horn[i] = True
    log['horn faces to bronze'] = int(horn.sum())
    shell_dark = [cmesh_q(hs[shell == j, 2], area[shell == j]) <= 16 for j in range(nshell)]      # whole dark insets: sockets, nostrils, grooves
    for i in range(nf):
        if mat[i] != 0 or fixed[i] or horn[i]: continue
        p = old_patch[i]; l = hs[i, 2]; n_ = nbl(p)
        if l <= 16 and (pn[p] >= 3 or shell_dark[shell[i]]):
            new[i] = pl.pack(*S[0 if l <= 10 else 1]); fixed[i] = True; log['dark stone kept'] += 1
        elif n_ is not None and l - n_ >= 12 and pn[p] >= 2 and pp.get(names[part[i]], {}).get('focal') and pa[p] < 0.004 * tot and hs[i, 1] <= 1:
            r = rng(i, 'stone') or [0, 9]; new[i] = pl.pack(*S[r[1] if l - n_ >= 20 else max(r[1] - 1, 0)]); fixed[i] = True; log['pale stone marks kept'] += 1
    # ---- 3. planes
    free = ~fixed
    plane = np.full(nf, -1, int); seeds = []
    part = part.copy(); names = list(names) + ['horn']; part[horn] = len(names) - 1          # horns: planes of their own
    for i in np.argsort(-area):
        if not free[i] or plane[i] >= 0: continue
        pid = len(seeds); seeds.append(i); plane[i] = pid; stack = [i]
        while stack:
            a = stack.pop()
            for b, k in nb[a]:
                if plane[b] >= 0 or not free[b] or shell[b] != shell[i] or part[b] != part[i]: continue
                foc = pp.get(names[part[i]], {}).get('focal', True)          # calm parts: larger planes, fewer boundaries
                if fold[k] < (16 if foc else 26) and float(unit[b] @ unit[i]) > math.cos(math.radians(28 if foc else 45)): plane[b] = pid; stack.append(b)
    npl = len(seeds)
    p_area = np.bincount(plane[free], weights=area[free], minlength=npl); p_n = np.bincount(plane[free], minlength=npl)
    conv = np.zeros(npl); conc = np.zeros(npl); per = np.zeros(npl)
    for k, (a, b) in enumerate(zip(ea, eb)):
        for x, y in ((a, b), (b, a)):
            if plane[x] >= 0 and plane[x] != plane[y]:
                per[plane[x]] += el[k]
                if fold[k] >= 20: (conv if convex[k] else conc)[plane[x]] += el[k]
    pc = np.zeros((npl, 3))
    for j in range(3): pc[:, j] = np.bincount(plane[free], weights=(cen[:, j] * area)[free], minlength=npl) / np.maximum(p_area, 1e-9)
    sharp_nb = collections.defaultdict(collections.Counter)
    for k, (a, b) in enumerate(zip(ea, eb)):
        if plane[a] >= 0 and plane[b] >= 0 and plane[a] != plane[b] and fold[k] >= 30: sharp_nb[plane[a]][plane[b]] += el[k]; sharp_nb[plane[b]][plane[a]] += el[k]
    tone = np.full(npl, -1, int)
    for q in np.argsort(-p_area):                    # large planes first; a plane avoids the tone of a plane it meets at a fold
        i = seeds[q]; pn_ = names[part[i]]
        if pn_ == 'horn':
            key = f"{seed}|h|{round(abs(pc[q, 0]) / 16)}|{round(pc[q, 1] / 16)}|{round(pc[q, 2] / 16)}"
            tone[q] = 100 + 2 + hashlib.md5(key.encode()).digest()[0] % 3; continue
        spec = pp.get(pn_, pp['trunk']); lo, hi = spec['ramps']['stone']
        t0 = spec['base'] + (1 if shell[i] != body else 0)
        relief = 0
        if per[q] > 0:
            if conv[q] / per[q] > 0.55: relief = 1            # a raised plane: paler
            elif conc[q] / per[q] > 0.45: relief = -1         # a recess: darker
        key = f"{seed}|{round(abs(pc[q, 0]) / 16)}|{round(pc[q, 1] / 16)}|{round(pc[q, 2] / 16)}"
        hsh = hashlib.md5(key.encode()).digest()
        amp = spec.get('plane_steps', 2 if spec.get('focal') else 1)          # plane tones stay near the part's base: soft steps
        first = max(-amp, min(amp, relief + ((-1, 0, 1, 0, -1, 1) if amp > 1 else (0, 0, 1, 0, -1, 0))[hsh[0] % 6]))
        if relief > 0: first = 2 if (amp < 2 and t0 + 2 < len(S) and S[t0 + 2][2] - S[t0][2] <= 7) else 1     # raised planes carry the light edge, kept under a mark's contrast
        cands = [first] + [o for o in ((0, 1, -1, 2, -2) if hsh[1] % 2 else (0, -1, 1, -2, 2)) if o != first and abs(o) <= amp]
        clampt = lambda o: min(max(t0 + o, max(lo, 2)), min(hi, plan.get('plane_top', 7)))
        best = None
        for rank, o in enumerate(cands):
            t = clampt(o); clash = sum(w for r, w in sharp_nb[q].items() if tone[r] == t)
            if best is None or clash + 1e-6 < best[0]: best = (clash, t)
            if clash == 0 or (amp < 2 and hsh[2] % 2): break   # calm parts: half the planes take their first choice (fewer boundaries)
        tone[q] = best[1]
    # ---- 4. tiny planes follow their largest neighbour
    small = (p_n < 2) & (p_area < 0.0006 * tot)
    share = collections.defaultdict(collections.Counter)
    for k, (a, b) in enumerate(zip(ea, eb)):
        if plane[a] >= 0 and plane[b] >= 0 and plane[a] != plane[b]: share[plane[a]][plane[b]] += el[k]; share[plane[b]][plane[a]] += el[k]
    for q in np.nonzero(small)[0]:
        big = [(w, r) for r, w in share[q].items() if not small[r]]
        if big: tone[q] = tone[max(big)[1]]; log['tiny planes merged'] += 1
    for i in range(nf):
        if free[i]: new[i] = pl.pack(*(B_[tone[plane[i]] - 100] if tone[plane[i]] >= 100 else S[tone[plane[i]]]))
    # ---- 5. left and right agree: a +x face takes the paint of its mirror twin on the -x (lit) side, stone only
    from scipy.spatial import cKDTree
    stone_cols = {pl.pack(*c) for c in S}
    neg = np.nonzero(cen[:, 0] < -1)[0]; tree = cKDTree(cen[neg])
    for i in np.nonzero(cen[:, 0] > 1)[0]:
        trunk = names[part[i]] == 'trunk'                 # the trunk keeps its one allowed asymmetry (the bronze repair, the belt)
        if trunk and int(new[i]) not in stone_cols: continue
        d_, j = tree.query([-cen[i, 0], cen[i, 1], cen[i, 2]]); j = neg[j]
        if d_ <= 10 and (int(new[j]) in stone_cols or not trunk) and new[j] != new[i] and float(unit[i] @ (unit[j] * np.array([-1, 1, 1]))) > 0.8: new[i] = new[j]; log['mirrored from the lit side'] += 1
    # ---- 6. no splinters: a plane-toned face unlike all its neighbours takes the paint of its longest stone neighbour
    for _ in range(2):
        for i in range(nf):
            if not free[i] or int(new[i]) not in stone_cols or not nb[i]: continue
            if all(new[b] != new[i] for b, k in nb[i]):
                c = [(el[k], int(new[b])) for b, k in nb[i] if int(new[b]) in stone_cols and free[b]]
                if c: new[i] = max(c)[1]; log['splinters absorbed'] += 1
    log['planes'] = int(npl); log['faces toned by plane'] = int(free.sum())
    cols = list(m['colors'])
    for i in range(nf): cols[int(idx[i])] = int(new[i])
    m['colors'] = cols
    # ---- optional: light edge strips on plate borders (re-split)
    made = 0
    if strips:
        made = edge_strips(m, sk, shell, body, part, names, plan, strips)
        log['edge strips'] = made
    d = HERE / 'repaint'; d.mkdir(exist_ok=True)
    meshio.save(m, d / f'{out}.bin')
    j = dict(base_def); j['models'] = [f'{out}.bin']; j['name'] = 'Meridian Watcher (repaint)'
    (d / f'{out}.json').write_text(json.dumps(j, indent=1)); (d / f'{out}-log.json').write_text(json.dumps(dict(log), indent=1))
    print(dict(log)); print('faces', len(m['faces']), 'vertices', len(m['vertices']), '->', d / f'{out}.json')
    return d / f'{out}.json'

def edge_strips(m, sk, shell, body, part, names, plan, limit, width=5.0):
    """On the stone plates (shells other than the body), a face with one edge on the plate's OPEN border and stone
    paint gets a strip `width` units wide along that border, painted two ramp steps paler: the worn edge. The two
    other edges of the face are split (their neighbours with them), so the strip has its own boundary."""
    S = plan['materials']['stone']['ramp']; sidx = {pl.pack(*c): i for i, c in enumerate(S)}
    F = sk['F']; idx = sk['idx']; V = sk['V']
    count = collections.Counter()
    for f in F:
        for a, b in ((f[0], f[1]), (f[1], f[2]), (f[2], f[0])): count[(min(a, b), max(a, b))] += 1
    crest = set()                                       # the plates here are closed shells: their border is the crest
    for k in range(len(sk['e_a'])):                     # where two stone faces meet at a convex fold of 45 degrees or more
        if sk['e_fold'][k] >= 45 and sk['e_convex'][k] and m['colors'][int(idx[sk['e_a'][k]])] in sidx and m['colors'][int(idx[sk['e_b'][k]])] in sidx:
            crest.add((int(min(sk['e_v'][k])), int(max(sk['e_v'][k]))))
    cand = []
    for i, f in enumerate(F):
        on_head = shell[i] == body and names[part[i]] == 'head' and abs(sk['cen'][i, 0]) < 60
        if not on_head and (shell[i] == body or (shell == shell[i]).sum() < 100): continue
        if m['colors'][int(idx[i])] not in sidx: continue
        for a, b in ((f[0], f[1]), (f[1], f[2]), (f[2], f[0])):
            if count[(min(a, b), max(a, b))] == 1 or (min(a, b), max(a, b)) in crest:
                c = [v for v in f if v != a and v != b][0]
                h = 2 * sk['area'][i] / max(np.linalg.norm(V[a] - V[b]), 1e-9)
                if h >= 2.2 * width and np.linalg.norm(V[a] - V[b]) >= 8: cand.append((np.linalg.norm(V[a] - V[b]), int(idx[i]), int(a), int(b), int(c), h, on_head, plan['paint'].get(names[part[i]], {}).get('ramps', {}).get('stone', [0, 9])[1]))
    cand.sort(reverse=True); made = 0; touched = set()
    quota = {True: limit, False: 0}                    # light worn edges belong to the focal head only (calm limbs stay calm)
    for _, fi, a, b, c, h, grp, top in cand:
        if quota[grp] <= 0: continue
        if {a, b, c} & touched: continue
        f = m['faces'][fi]
        if not (a in f and b in f and c in f): continue
        col = m['colors'][fi]
        if col not in sidx or sidx[col] + 2 > top: continue
        lite = pl.pack(*S[sidx[col] + 2])
        t = width / h
        p = split_edge(m, a, c, t); q = split_edge(m, b, c, t)
        if p is None or q is None: continue
        touched |= {a, b, c}
        for k, f2 in enumerate(m['faces']):
            if set(f2) <= {a, b, c, p, q} and c not in f2: m['colors'][k] = lite
        made += 1; quota[grp] -= 1
    return made

if __name__ == '__main__':
    a = sys.argv[1:]
    def opt(n, d=None): return a[a.index(n) + 1] if n in a else d
    run(opt('--out', 'watcher-2-repaint'), int(opt('--strips', 0)), opt('--seed', 'meridian'))
