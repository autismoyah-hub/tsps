"""The 17 pre-registered measures of PREREG.md, exactly as defined there.

    P measures.py                 all 41 creatures -> data/measures.json
    P measures.py NAME|ENTRY ..   print the measures of some
`measure(c, scale)` takes a cleanlib creature dict; used by gapcheck.py.
"""
import sys, pathlib, json, math, collections
import numpy as np, cv2
from scipy.spatial import cKDTree
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import gaplib as g
cl = g.cl

KEYS = ['flow_aligned', 'ring_cv', 'plane_by_part', 'big_planes', 'sil_cut', 'join_line', 'crease_chain', 'dangling',
        'concave', 'relief', 'mark_seat', 'subpixel', 'asym', 'crisp_close', 'define_game', 'depth_close', 'sil_turn']
DIRECTION = {k: 'higher' for k in KEYS}
DIRECTION.update(dangling='lower', subpixel='lower', asym='lower')

class UF:
    def __init__(s, n): s.p = np.arange(n)
    def find(s, x):
        p = s.p
        while p[x] != x: p[x] = p[p[x]]; x = p[x]
        return x
    def union(s, a, b):
        a, b = s.find(a), s.find(b)
        if a != b: s.p[b] = a
    def roots(s): return np.array([s.find(i) for i in range(len(s.p))])

def prep(c):
    cl.topology(c)
    skin = c['drawn'] & (c['A'] == 0)
    c['skin'] = skin
    ok = skin[c['e_a']] & skin[c['e_b']]
    c['E'] = np.nonzero(ok)[0]
    sf = np.nonzero(skin)[0]
    used = np.unique(c['F'][sf])
    c['H'] = float(np.ptp(c['V'][used][:, 1])) or 1.0
    fold = np.abs(c['e_dih']); c['fold'] = fold
    c['line'] = (c['C'][c['e_a']] != c['C'][c['e_b']]) | (fold >= 30)
    return c

def kmeans_parts(c, k=16):
    sf = np.nonzero(c['skin'])[0]
    X = c['cen'][sf] / c['H']; w = c['area'][sf]
    rng = np.random.default_rng(0)
    cent = X[rng.choice(len(sf), size=min(k, len(sf)), replace=False, p=w / w.sum())]
    for _ in range(30):
        lab = np.argmin(((X[:, None, :] - cent[None]) ** 2).sum(2), 1)
        cent = np.array([np.average(X[lab == j], axis=0, weights=w[lab == j]) if (lab == j).any() else cent[j] for j in range(len(cent))])
    part = np.full(len(c['F']), -1); part[sf] = lab
    return part

def elongated(c, part):
    out = {}
    for j in np.unique(part[part >= 0]):
        fs = np.nonzero(part == j)[0]
        if len(fs) < 6: continue
        w = c['area'][fs]; X = c['cen'][fs]
        mu = np.average(X, axis=0, weights=w); D = X - mu
        cov = (D * w[:, None]).T @ D / w.sum()
        ev, evec = np.linalg.eigh(cov)
        if ev[1] <= 0: continue
        if math.sqrt(ev[2] / ev[1]) >= 2: out[int(j)] = (evec[:, 2], fs, float(w.sum()))
    return out

def m_flow_ring(c, part, el):
    E = c['E']; ea, eb, ev = c['e_a'][E], c['e_b'][E], c['e_v'][E]
    tot = 0.0; good = 0.0; cvs = []; ws = []
    for j, (ax, fs, area) in el.items():
        sel = (part[ea] == j) & (part[eb] == j)
        if sel.any():
            vec = c['V'][ev[sel, 1]] - c['V'][ev[sel, 0]]; L = np.linalg.norm(vec, axis=1)
            cs = np.abs(vec @ ax) / np.maximum(L, 1e-9)
            ok = (cs >= math.cos(math.radians(25))) | (cs <= math.cos(math.radians(65)))
            tot += L.sum(); good += L[ok].sum()
        vs = np.unique(c['F'][fs]); t = np.sort(c['V'][vs] @ ax)
        span = t[-1] - t[0]
        if span <= 0: continue
        brk = np.nonzero(np.diff(t) > 0.02 * span)[0]
        groups = np.split(t, brk + 1)
        if len(groups) >= 4:
            cen = np.array([x.mean() for x in groups]); gaps = np.diff(cen)
            cvs.append(gaps.std() / max(gaps.mean(), 1e-9)); ws.append(area)
    flow = good / tot if tot > 0 else None
    if cvs:
        o = np.argsort(cvs); cw = np.cumsum(np.array(ws)[o]) / sum(ws)
        ring = float(np.array(cvs)[o][np.searchsorted(cw, 0.5)])
    else: ring = None
    return flow, ring

def m_plane_by_part(c, part):
    meds = [np.median(c['area'][part == j]) for j in np.unique(part[part >= 0]) if (part == j).sum() >= 1]
    meds = np.array(meds); p10 = np.quantile(meds, .1)
    return float(np.log10(np.quantile(meds, .9) / p10)) if p10 > 0 else None

def m_big_planes(c):
    nf = len(c['F']); uf = UF(nf); E = c['E']
    for k in E[~c['line'][E]]: uf.union(int(c['e_a'][k]), int(c['e_b'][k]))
    sf = np.nonzero(c['skin'])[0]; r = np.array([uf.find(int(i)) for i in sf])
    a = c['area'][sf]; tot = a.sum()
    sums = collections.defaultdict(float)
    for ri, ai in zip(r, a): sums[ri] += ai
    return float(sum(v for v in sums.values() if v > 0.02 * tot) / tot)

def m_sil_cut(c):
    E = c['E']; ua, ub = c['unit'][c['e_a'][E]], c['unit'][c['e_b'][E]]; L = c['e_len'][E]; fold = c['fold'][E]
    tot = 0.0; hard = 0.0
    for yaw in range(0, 360, 45):
        d = cl.camera(yaw, 0)[2]
        cont = np.sign(ua @ d) != np.sign(ub @ d)
        tot += L[cont].sum(); hard += L[cont & (fold >= 50)].sum()
    return hard / tot if tot > 0 else None

def m_join_line(c):
    Lb = c['L']
    if len(set(Lb.tolist())) <= 1: return None
    F = c['F']; fl = np.empty(len(F), int)
    for i, f in enumerate(F):
        a, b, d = sorted(int(Lb[v]) for v in f)
        fl[i] = a if (a == b or a == d) else (b if b == d else a)   # commonest; all different -> lowest
    E = c['E']; diff = fl[c['e_a'][E]] != fl[c['e_b'][E]]
    L = c['e_len'][E]
    if L[diff].sum() <= 0: return None
    return float(L[diff & c['line'][E]].sum() / L[diff].sum())

def m_chains(c):
    E = c['E']; hard = E[c['fold'][E] >= 50]
    if len(hard) == 0: return None, None
    V = c['V']; ev = c['e_v']
    at = collections.defaultdict(list)
    for k in hard: at[int(ev[k, 0])].append(int(k)); at[int(ev[k, 1])].append(int(k))
    openv = set()
    for a, b in c['open']: openv.add(int(a)); openv.add(int(b))
    seen = set(); chains = []
    def other(k, v): return int(ev[k, 1]) if int(ev[k, 0]) == v else int(ev[k, 0])
    def extend(k, v):
        """walk from edge k leaving through vertex v; returns list of edges and the end vertex"""
        out = []; cur = k; prev_v = other(k, v)
        while True:
            dirin = V[v] - V[prev_v]; dirin = dirin / max(np.linalg.norm(dirin), 1e-9)
            best = None; bang = 999
            for k2 in at[v]:
                if k2 == cur or k2 in seen: continue
                w = other(k2, v); d2 = V[w] - V[v]; d2 = d2 / max(np.linalg.norm(d2), 1e-9)
                ang = math.degrees(math.acos(max(-1, min(1, float(dirin @ d2)))))
                if ang < bang: bang, best = ang, k2
            if best is None or bang > 35: return out, v
            seen.add(best); out.append(best); prev_v = v; v = other(best, v); cur = best
    ends = 0
    for k in hard:
        k = int(k)
        if k in seen: continue
        seen.add(k)
        a, b = int(ev[k, 0]), int(ev[k, 1])
        e1, va = extend(k, a); e2, vb = extend(k, b)
        ch = [k] + e1 + e2; chains.append(float(c['e_len'][ch].sum()))
        for v in (va, vb):
            if len(at[v]) == 1 and v not in openv: ends += 1
    hl = float(c['e_len'][hard].sum()); H = c['H']
    share = sum(x for x in chains if x >= 0.10 * H) / hl
    return float(share), float(ends / (hl / H))

def m_concave(c):
    E = c['E']; same = c['C'][c['e_a'][E]] == c['C'][c['e_b'][E]]
    L = c['e_len'][E]; dih = c['e_dih'][E]
    if L[same].sum() <= 0: return None
    return float(L[same & (dih <= -30)].sum() / L[same].sum())

def m_relief(c):
    sf = np.nonzero(c['skin'])[0]; F = c['F'][sf]
    vs = np.unique(F); idx = {int(v): i for i, v in enumerate(vs)}
    nb = collections.defaultdict(set)
    for f in F:
        for a, b in ((f[0], f[1]), (f[1], f[2]), (f[2], f[0])): nb[idx[int(a)]].add(idx[int(b)]); nb[idx[int(b)]].add(idx[int(a)])
    X0 = c['V'][vs].astype(float); X = X0.copy()
    rows = np.concatenate([[i] * len(nb[i]) for i in range(len(vs))]).astype(int)
    cols = np.concatenate([list(nb[i]) for i in range(len(vs))]).astype(int)
    deg = np.bincount(rows, minlength=len(vs)).astype(float)
    for _ in range(10):
        m = np.zeros_like(X); np.add.at(m, rows, X[cols]); m /= np.maximum(deg, 1)[:, None]
        X = X + 0.5 * (m - X)
    w = np.zeros(len(vs)); a = c['area'][sf]
    for k in range(3): np.add.at(w, [idx[int(v)] for v in F[:, k]], a / 3)
    d = np.linalg.norm(X - X0, axis=1)
    return float((d * w).sum() / w.sum() / c['H'])

def m_mark_seat(c):
    sf = np.nonzero(c['skin'])[0]; skin = c['skin']; F = c['F']; C = c['C']; A = c['area']
    Lf = c['LC'].mean(1).astype(float)
    E = c['E']; uf = UF(len(F))
    same = C[c['e_a'][E]] == C[c['e_b'][E]]
    for k in E[same]: uf.union(int(c['e_a'][k]), int(c['e_b'][k]))
    comp = collections.defaultdict(list)
    for i in sf: comp[uf.find(int(i))].append(int(i))
    vf = c['vf']; tot = A[sf].sum()
    def ring(faces, excl):
        r = set()
        for f in faces:
            for v in F[f]:
                for f2 in vf[int(v)]:
                    if skin[f2] and f2 not in excl: r.add(f2)
        return r
    def mean(fs):
        fs = np.array(sorted(fs)); w = A[fs]
        return float((Lf[fs] * w).sum() / max(w.sum(), 1e-9))
    n = 0; seated = 0
    for faces in comp.values():
        if A[faces].sum() > 0.005 * tot: continue
        ms = set(faces); r1 = ring(faces, ms)
        if not r1: continue
        lm, l1 = mean(ms), mean(r1)
        if abs(lm - l1) < 12: continue
        r2 = ring(r1, ms | r1)
        if not r2: continue
        n += 1; seated += abs(l1 - mean(r2)) >= 4
    return (seated / n if n >= 3 else None), n

def m_subpixel(c, scale):
    ppu = 0.445 * scale / 128; sf = np.nonzero(c['skin'])[0]
    return float((c['area'][sf] * ppu * ppu < 0.5).mean())

def m_asym(c):
    sf = np.nonzero(c['skin'])[0]; a = c['area'][sf]; rng = np.random.default_rng(0)
    pick = sf[rng.choice(len(sf), size=800, p=a / a.sum())]
    u, v = rng.random(800), rng.random(800); flip = u + v > 1; u[flip], v[flip] = 1 - u[flip], 1 - v[flip]
    T = c['V'][c['F'][pick]]
    P = T[:, 0] + u[:, None] * (T[:, 1] - T[:, 0]) + v[:, None] * (T[:, 2] - T[:, 0])
    used = np.unique(c['F'][sf]); xs = c['V'][used][:, 0]; xc = (xs.min() + xs.max()) / 2
    M = P.copy(); M[:, 0] = 2 * xc - M[:, 0]
    d, _ = cKDTree(P).query(M)
    return float(d.mean() / c['H'])

def pictures(c, scale):
    skin = c['skin']; out = collections.defaultdict(list)
    for yaw in (0, 45, 90):
        for mode in ('close', 'game'):
            if mode == 'close': img, ids = cl.view(c, yaw=yaw, pitch=10, size=420)
            else: img, ids = cl.view(c, yaw=yaw, pitch=39, ppu=0.445 * scale / 128, pad=4)
            gr = cl.grey(img); body = (ids >= 0) & skin[np.maximum(ids, 0)]
            dh = np.abs(gr[:, 1:] - gr[:, :-1]); bh = body[:, 1:] & body[:, :-1]
            dv = np.abs(gr[1:] - gr[:-1]); bv = body[1:] & body[:-1]
            d = np.concatenate([dh[bh], dv[bv]])
            if mode == 'close':
                s12 = (d >= 12).sum()
                out['crisp_close'].append((d >= 40).sum() / s12 if s12 else 0.0)
                gb = gr[body]
                out['depth_close'].append(float(np.quantile(gb, .9) - np.quantile(gb, .1)) if gb.size else 0.0)
                mask = (ids >= 0).astype(np.uint8) * 255
                cs, _ = cv2.findContours(mask, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_NONE)
                if cs:
                    cnt = max(cs, key=cv2.contourArea); poly = cv2.approxPolyDP(cnt, 1.5, True)[:, 0, :].astype(float)
                    if len(poly) >= 3:
                        seg = np.roll(poly, -1, 0) - poly; ang = np.arctan2(seg[:, 1], seg[:, 0])
                        turn = np.abs((np.roll(ang, -1) - ang + np.pi) % (2 * np.pi) - np.pi)
                        out['sil_turn'].append(float(turn[turn >= math.radians(45)].sum() / max(turn.sum(), 1e-9)))
            else:
                strong = np.zeros_like(body)
                hh = (dh >= 12) & bh; vv = (dv >= 12) & bv
                strong[:, 1:] |= hh; strong[:, :-1] |= hh; strong[1:] |= vv; strong[:-1] |= vv
                nb = body.sum()
                out['define_game'].append(float((strong & body).sum() / nb) if nb else 0.0)
    return {k: float(np.mean(v)) for k, v in out.items()}

def measure(c, scale=128, with_pictures=True):
    prep(c)
    part = kmeans_parts(c); el = elongated(c, part)
    m = {}
    m['flow_aligned'], m['ring_cv'] = m_flow_ring(c, part, el)
    m['plane_by_part'] = m_plane_by_part(c, part)
    m['big_planes'] = m_big_planes(c)
    m['sil_cut'] = m_sil_cut(c)
    m['join_line'] = m_join_line(c)
    m['crease_chain'], m['dangling'] = m_chains(c)
    m['concave'] = m_concave(c)
    m['relief'] = m_relief(c)
    m['mark_seat'], m['n_marks'] = m_mark_seat(c)
    m['subpixel'] = m_subpixel(c, scale)
    m['asym'] = m_asym(c)
    if with_pictures: m.update(pictures(c, scale))
    m['n_elongated_parts'] = len(el); m['H'] = c['H']; m['skin_faces'] = int(c['skin'].sum())
    return m

def scale_of(entry):
    if entry.startswith('npc:'): return cl.census()[int(entry[4:])]['scale'][0]
    return json.loads(pathlib.Path(entry[4:]).read_text()).get('scale', [128, 128])[0]

if __name__ == '__main__':
    names = sys.argv[1:] or list(g.ALL)
    out = {}
    for n in names:
        e = g.ALL.get(n, n)
        c = g.load(cl.norm_entry(e) if n not in g.ALL else e)
        m = measure(c, scale_of(cl.norm_entry(e)))
        out[n] = dict(group=g.GROUP.get(n, '?'), m=m)
        print(n, out[n]['group'], 'done', flush=True)
    if not sys.argv[1:]:
        (HERE / 'data' / 'measures.json').write_text(json.dumps(out, indent=1))
        print(HERE / 'data' / 'measures.json')
    else:
        for n, v in out.items(): print(n, json.dumps({k: (round(x, 4) if isinstance(x, float) else x) for k, x in v['m'].items()}))
