"""Exploration 3: small planes / patches split into WEAK (splotch candidates) and STRONG (marks). -> data/explore3.json"""
import sys, pathlib, json, math, collections
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
import cleanlib as cl
from explore import all_entries, same_mat
HERE = pathlib.Path(__file__).resolve().parent

def regions(c, link):
    """union faces over edges where link[k] is True; returns root per face (dict)"""
    skin = c['skinmask']
    parent = {int(i): int(i) for i in np.nonzero(skin)[0]}
    def find(x):
        while parent[x] != x: parent[x] = parent[parent[x]]; x = parent[x]
        return x
    for x, y, l in zip(c['sea'], c['seb'], link):
        if l: parent[find(int(x))] = find(int(y))
    return {i: find(i) for i in parent}

def prep(c):
    cl.topology(c)
    F, C, V, LC = c['F'], c['C'], c['V'], c['LC'].astype(float); area = c['area']
    skin = c['drawn'] & (c['A'] == 0); c['skinmask'] = skin
    used = np.unique(F[skin]); c['hgt'] = float(np.ptp(V[used][:, 1])); c['tot'] = float(area[skin].sum())
    ea, eb = c['e_a'], c['e_b']; both = skin[ea] & skin[eb]
    c['sea'], c['seb'], c['sel'], c['sev'], c['sdih'] = ea[both], eb[both], c['e_len'][both], c['e_v'][both], c['e_dih'][both]
    ea, eb, ev = c['sea'], c['seb'], c['sev']
    a, b, d = V[F[:, 0]], V[F[:, 1]], V[F[:, 2]]
    e1, e2 = b - a, d - a
    n = np.cross(e1, e2); nn = (n * n).sum(1); nn[nn == 0] = 1
    l0, l1, l2 = LC[:, 0], LC[:, 1], LC[:, 2]
    grad = ((l1 - l0)[:, None] * np.cross(e2, n) + (l2 - l0)[:, None] * np.cross(n, e1)) / nn[:, None]
    ed = V[ev[:, 1]] - V[ev[:, 0]]; ed /= np.maximum(np.linalg.norm(ed, axis=1), 1e-9)[:, None]
    ta = np.cross(c['unit'][ea], ed); tb = np.cross(c['unit'][eb], ed)
    dc = c['cen'][eb] - c['cen'][ea]
    sa = np.sign((dc * ta).sum(1)); sb = np.sign((dc * tb).sum(1))
    c['kink'] = np.abs((grad[ea] * ta).sum(1) * sa - (grad[eb] * tb).sum(1) * sb) * 0.02 * c['hgt']
    c['samecol'] = C[ea] == C[eb]
    c['sm'] = np.array([same_mat(int(x), int(y)) for x, y in zip(C[ea], C[eb])])
    c['litf'] = LC.mean(1)
    return c

def small_regions(c, root, small=0.005, weak=6.0, use='lit'):
    area = c['area']; tot = c['tot']; C = c['C']
    val = c['litf'] if use == 'lit' else (C & 127).astype(float)
    ra = collections.Counter(); rv = collections.Counter(); rf = collections.Counter()
    for i, r in root.items(): ra[r] += area[i]; rv[r] += area[i] * val[i]; rf[r] += 1
    nb = collections.defaultdict(lambda: [0.0, 0.0, 0.0])     # border length, sum(len * neighbour face value), len to another material
    for x, y, l, s in zip(c['sea'], c['seb'], c['sel'], c['sm']):
        rx, ry = root[int(x)], root[int(y)]
        if rx == ry: continue
        nb[rx][0] += l; nb[rx][1] += l * val[y]; nb[ry][0] += l; nb[ry][1] += l * val[x]
        if not s: nb[rx][2] += l; nb[ry][2] += l
    out = dict(weak=0.0, strong=0.0, accent=0.0, nweak=0, nstrong=0, weak1=0.0)
    kinds = {}
    for r, a in ra.items():
        if a > small * tot or nb[r][0] == 0: continue
        con = abs(rv[r] / a - nb[r][1] / nb[r][0]); acc = nb[r][2] / nb[r][0] >= 0.7
        if acc: out['accent'] += a / tot; kinds[r] = 'accent'
        elif con >= weak: out['strong'] += a / tot; out['nstrong'] += 1; kinds[r] = 'strong'
        else:
            out['weak'] += a / tot; out['nweak'] += 1; kinds[r] = 'weak'
            if rf[r] == 1: out['weak1'] += a / tot
    return out, kinds

def measure(c):
    prep(c); out = {}
    nf = int(c['skinmask'].sum())
    # (1) paint patches
    root = regions(c, c['samecol'])
    for small in (0.002, 0.005):
        r, _ = small_regions(c, root, small, 6.0, 'paint'); tag = f'pp{int(small * 1000)}'
        out[tag + '_weak'] = r['weak']; out[tag + '_strong'] = r['strong'] + r['accent']; out[tag + '_weak_n100'] = 100 * r['nweak'] / nf
        out[tag + '_weak_frac'] = r['weak'] / max(r['weak'] + r['strong'] + r['accent'], 1e-9)
    # (2) lit planes (paint equal and no light kink of 4 or more)
    root = regions(c, c['samecol'] & (c['kink'] < 4))
    for small in (0.002, 0.005):
        r, _ = small_regions(c, root, small, 6.0, 'lit'); tag = f'lp{int(small * 1000)}'
        out[tag + '_weak'] = r['weak']; out[tag + '_strong'] = r['strong'] + r['accent']; out[tag + '_weak_n100'] = 100 * r['nweak'] / nf
        out[tag + '_weak_frac'] = r['weak'] / max(r['weak'] + r['strong'] + r['accent'], 1e-9); out[tag + '_weak1'] = r['weak1']
    # (3) kinks by the fold that makes them: light lines on soft / mush / firm / hard folds within one colour
    k = c['samecol'] & (c['kink'] >= 4); el = c['sel']; ad = np.abs(c['sdih'])
    tot = el.sum()
    out['kinkline_soft'] = float(el[k & (ad < 30)].sum() / tot); out['kinkline_firm'] = float(el[k & (ad >= 30) & (ad < 50)].sum() / tot); out['kinkline_hard'] = float(el[k & (ad >= 50)].sum() / tot)
    out['hard_over_firm'] = float(el[c['sm'] & (ad >= 50)].sum() / max(el[c['sm'] & (ad >= 30) & (ad < 50)].sum(), 1e-9))
    out['hard_over_firm_all'] = float(el[(ad >= 50)].sum() / max(el[(ad >= 30) & (ad < 50)].sum(), 1e-9))
    return out

if __name__ == '__main__':
    store = HERE / 'data' / 'explore3.json'
    rows = json.loads(store.read_text()) if store.exists() and '--refresh' not in sys.argv else {}
    for key, entry, name, st in all_entries():
        if key in rows: continue
        c = cl.load(entry); r = measure(c); r['name'] = name; r['style'] = st; rows[key] = r
        store.write_text(json.dumps(rows, indent=0))
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
