"""Exploration 5 ('well defined'): creases and strong lines as continuous chains; head measures; feature borders."""
import sys, pathlib, json, collections, math
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
import cleanlib as cl
from explore import all_entries
from explore3 import prep
HERE = pathlib.Path(__file__).resolve().parent
HEADS = {'12812': (0.78, 1.0), '2025': (0.66, 0.86), '3129': (0.72, 0.95), '2215': (0.84, 1.0), '13599': (0.78, 0.9), '14176': (0.84, 1.0), '7144': (0.84, 1.0), '7286': (0.8, 1.0),
         '7541': (0.8, 1.0), '11278': (0.88, 1.0), '7849': (0.85, 1.0), 'watcher-2': (0.74, 1.0), 'watcher-repaint': (0.74, 1.0), 'warden-general-3': (0.85, 1.0)}

def chains(c, sel):
    """connected groups of the selected edges (through shared vertices); returns list of (length, n_edges, straightness)"""
    ev, el = c['sev'][sel], c['sel'][sel]
    parent = {}
    def find(x):
        while parent.setdefault(x, x) != x: parent[x] = parent[parent[x]]; x = parent[x]
        return x
    for a, b in ev: parent[find(int(a))] = find(int(b))
    L = collections.Counter(); N = collections.Counter()
    for (a, b), l in zip(ev, el): r = find(int(a)); L[r] += l; N[r] += 1
    return L, N

def measure(c, key):
    prep(c); out = {}
    h = c['hgt']; el = c['sel']; ad = np.abs(c['sdih']); tot = el.sum()
    pl = (c['C'] & 127).astype(float)
    step = np.abs(pl[c['sea']] - pl[c['seb']]) * (c['LN'][c['sev']].mean(1) / 128)
    hard = ad >= 50
    strong = (step >= 6) | (~c['sm'])
    for name, sel in (('hard', hard), ('strong', strong), ('def', hard | strong)):
        L, N = chains(c, sel)
        ls = np.array(list(L.values())) if L else np.zeros(1)
        s = ls.sum()
        out[f'{name}_len_per_h'] = float(s / h)
        out[f'{name}_density'] = float(s / math.sqrt(c['tot']))
        out[f'{name}_in_long'] = float(ls[ls >= 0.15 * h].sum() / max(s, 1e-9))
        out[f'{name}_in_short'] = float(ls[ls < 0.04 * h].sum() / max(s, 1e-9))
        out[f'{name}_single_edges'] = float(sum(L[r] for r in L if N[r] == 1) / max(s, 1e-9))
    # agreement of form and paint: a hard fold that also carries a tone step; a strong line that sits on a fold
    out['hard_with_step'] = float(el[hard & (step >= 2)].sum() / max(el[hard].sum(), 1e-9))
    out['strong_on_fold30'] = float(el[strong & (ad >= 30)].sum() / max(el[strong].sum(), 1e-9))
    out['strong_on_flat10'] = float(el[strong & (ad < 10)].sum() / max(el[strong].sum(), 1e-9))
    # firm folds (30-50): do they carry paint? A fold that is neither committed nor painted is the 'mushy' fold.
    firm = (ad >= 30) & (ad < 50)
    out['firm_unpainted'] = float(el[firm & c['samecol']].sum() / tot)
    out['hard_unpainted'] = float(el[hard & c['samecol']].sum() / tot)
    out['mush_unpainted_over_hard'] = float(el[firm & c['samecol']].sum() / max(el[hard].sum(), 1e-9))
    if key in HEADS:
        lo, hi = HEADS[key]; V = c['V']; skin = c['skinmask']; used = np.unique(c['F'][skin]); top = V[used][:, 1].max()
        up = (top - c['cen'][:, 1]) / h
        hf = skin & (up >= lo) & (up <= hi); ha = c['area'][hf].sum()
        he = hf[c['sea']] & hf[c['seb']]
        out['head_faces'] = int(hf.sum()); out['head_face_share'] = float(hf.sum() / skin.sum()); out['head_area_share'] = float(ha / c['tot'])
        out['head_density_ratio'] = out['head_face_share'] / max(out['head_area_share'], 1e-9)
        out['head_hard_density'] = float(el[he & hard].sum() / math.sqrt(ha)); out['head_strong_density'] = float(el[he & strong].sum() / math.sqrt(ha))
        out['head_hard_over_firm'] = float(el[he & hard].sum() / max(el[he & firm].sum(), 1e-9))
        g = c['litf'][hf]; w = c['area'][hf]
        out['head_lit_span'] = cl.cmesh.wquant(g, w, .9) - cl.cmesh.wquant(g, w, .1)
        out['head_colours'] = int(len(set(c['C'][hf].tolist())))
    return out

if __name__ == '__main__':
    rows = {}
    for key, entry, name, st in all_entries():
        c = cl.load(entry); r = measure(c, key); r['name'] = name; r['style'] = st; rows[key] = r
    (HERE / 'data' / 'explore5.json').write_text(json.dumps(rows, indent=0))
    keys = [k for k in rows['12812'] if k not in ('name', 'style')]
    oursk = ['watcher-2', 'watcher-repaint', 'star-hound-3', 'warden-general-3']
    print(f"{'measure':26s} {'real min':>8s} {'p10':>7s} {'med':>7s} {'p90':>7s} {'max':>8s} | {'Mino':>7s} {'Tekton':>7s} | {'w2':>7s} {'w2-rep':>7s} {'hound3':>7s} {'WG3':>7s} | out")
    for k in keys:
        v = np.array([r[k] for r in rows.values() if r['style'] != 'o' and k in r], float)
        o = [rows[q].get(k, float('nan')) for q in oursk]
        flags = ''.join('?' if x != x else '<' if x < v.min() else '>' if x > v.max() else ('(' if x < np.quantile(v, .1) else ')' if x > np.quantile(v, .9) else '.') for x in o)
        print(f"{k:26s} {v.min():8.3f} {np.quantile(v, .1):7.3f} {np.median(v):7.3f} {np.quantile(v, .9):7.3f} {v.max():8.3f} | {rows['12812'][k]:7.3f} {rows['7541'][k]:7.3f} | " + ' '.join(f'{x:7.3f}' for x in o) + f' | {flags}  n={len(v)}')
