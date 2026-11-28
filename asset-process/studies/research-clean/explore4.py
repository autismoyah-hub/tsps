"""Exploration 4: 'unanchored' small weak planes: small lit planes that differ a little from their neighbours and whose
border is neither a committed fold nor a strong drawn line. Variants of the thresholds, per creature."""
import sys, pathlib, json, collections
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
import cleanlib as cl
from explore import all_entries
from explore3 import prep, regions
HERE = pathlib.Path(__file__).resolve().parent

def splotch(c, small=0.002, weak=6.0, hard=50.0, kink_t=4.0, anchor_share=0.34, strong_step=6.0):
    area = c['area']; tot = c['tot']; C = c['C']; val = c['litf']
    root = regions(c, c['samecol'] & (c['kink'] < kink_t))
    ra = collections.Counter(); rv = collections.Counter(); rf = collections.Counter()
    for i, r in root.items(): ra[r] += area[i]; rv[r] += area[i] * val[i]; rf[r] += 1
    nb = collections.defaultdict(lambda: [0.0, 0.0, 0.0, 0.0])   # border length, sum(len*neighbour lit), other-material len, anchored len
    pl = (C & 127).astype(float)
    step = np.abs(pl[c['sea']] - pl[c['seb']]) * (c['LN'][c['sev']].mean(1) / 128)
    anch = (np.abs(c['sdih']) >= hard) | (step >= strong_step) | (~c['sm'])
    for x, y, l, s, an in zip(c['sea'], c['seb'], c['sel'], c['sm'], anch):
        rx, ry = root[int(x)], root[int(y)]
        if rx == ry: continue
        for r, o in ((rx, y), (ry, x)):
            n = nb[r]; n[0] += l; n[1] += l * val[o]
            if not s: n[2] += l
            if an: n[3] += l
    kinds = {}
    out = dict(un=0.0, anchored=0.0, strong=0.0, n_un=0)
    for r, a in ra.items():
        n = nb[r]
        if a > small * tot or n[0] == 0: continue
        con = abs(rv[r] / a - n[1] / n[0])
        if n[2] / n[0] >= 0.7 or con >= weak: out['strong'] += a / tot; kinds[r] = 'mark'
        elif n[3] / n[0] >= anchor_share: out['anchored'] += a / tot; kinds[r] = 'facet'
        else: out['un'] += a / tot; out['n_un'] += 1; kinds[r] = 'splotch'
    return out, kinds, root

if __name__ == '__main__':
    variants = dict(base={}, small5=dict(small=0.005), hard40=dict(hard=40.0), anch50=dict(anchor_share=0.5), anch25=dict(anchor_share=0.25), kink3=dict(kink_t=3.0), kink6=dict(kink_t=6.0), weak4=dict(weak=4.0),
                    s5a50=dict(small=0.005, anchor_share=0.5), s1=dict(small=0.001))
    rows = {}
    for key, entry, name, st in all_entries():
        c = prep(cl.load(entry)); rows[key] = dict(name=name, style=st)
        for v, kw in variants.items():
            o, _, _ = splotch(c, **kw); rows[key][v] = o['un']; rows[key][v + '_facet'] = o['anchored']; rows[key][v + '_mark'] = o['strong']
    (HERE / 'data' / 'explore4.json').write_text(json.dumps(rows, indent=0))
    ks = list(variants)
    print(f"{'creature':20s} st " + ' '.join(f'{k:>7s}' for k in ks) + '   facet    mark')
    for k, r in sorted(rows.items(), key=lambda kv: kv[1]['base']):
        print(f"{r['name'][:20]:20s} {r['style']}  " + ' '.join(f'{r[q]:7.3f}' for q in ks) + f"  {r['base_facet']:7.3f} {r['base_mark']:7.3f}")
    for v in ks:
        real = [r[v] for r in rows.values() if r['style'] != 'o']
        print(v, 'real max', round(max(real), 3), 'second', round(sorted(real)[-2], 3), ' watcher-2', round(rows['watcher-2'][v], 3), ' repaint', round(rows['watcher-repaint'][v], 3), ' margin', round(min(rows['watcher-2'][v], rows['watcher-repaint'][v]) / max(real), 2))
