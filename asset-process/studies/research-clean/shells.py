"""List the shells (pieces connected by shared vertices) of a creature: faces, share of skin, colours, where, open edges,
and how much of the piece lies inside the main skin. usage: shells.py ENTRY [...]"""
import sys, pathlib, collections
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
import cleanlib as cl

def shell_table(c):
    cl.topology(c)
    F, V, C, area = c['F'], c['V'], c['C'], c['area']
    skin = c['drawn']
    groups = collections.defaultdict(list)
    for i in np.nonzero(skin)[0]: groups[c['shell'][int(i)]].append(int(i))
    used = np.unique(F[skin]); lo, hi = V[used].min(0), V[used].max(0); h = hi[1] - lo[1]
    openv = collections.Counter()
    for (a, b) in c['open']: openv[c['shell'][c['vf'][a][0]]] += 1
    rows = []
    for r, fs in groups.items():
        fs = np.array(fs); vs = np.unique(F[fs]); cen = V[vs].mean(0); ext = np.ptp(V[vs], axis=0)
        cols = collections.Counter(int(x) for x in C[fs]).most_common(3)
        rows.append(dict(root=r, faces=len(fs), area=float(area[fs].sum()), cen=cen, ext=ext, cols=[cl.hsl(k) for k, _ in cols], open=openv.get(r, 0), fs=fs,
                         up=float((hi[1] - cen[1]) / h), alpha=int(c['A'][fs].max()), labels=sorted(set(int(x) for x in c['L'][vs]))))
    rows.sort(key=lambda r: -r['faces'])
    return rows, h

if __name__ == '__main__':
    for e in sys.argv[1:]:
        c = cl.load(e); rows, h = shell_table(c); tot = sum(r['area'] for r in rows)
        print(f"== {c['name']}: {len(rows)} shells, height {h:.0f}")
        for r in rows:
            print(f"  faces {r['faces']:5d}  area {r['area'] / tot:6.3f}  at x {r['cen'][0]:6.0f} up {r['up']:.2f} z {r['cen'][2]:6.0f}  size {r['ext'][0]:4.0f} x {r['ext'][1]:4.0f} x {r['ext'][2]:4.0f}  open edges {r['open']:3d}  alpha {r['alpha']:3d}  labels {r['labels'][:6]}  colours {r['cols']}")
