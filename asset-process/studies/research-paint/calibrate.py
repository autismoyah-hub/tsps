"""How many of stylecheck's BAND lines each measured creature breaks (from data/paint.json; no plan needed).
usage: calibrate.py"""
import sys, pathlib, collections
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import measure_paint as mp, stylecheck as sc
d = mp.load(); res = {}
for k, v in d.items():
    f = w = 0; which = []
    for key, side, level, words in sc.BAND:
        b = mp.band(d, key); x = v['m'].get(key)
        if b is None or x is None: continue
        if key == 'mirror_paint_near' and v['m']['mirror_geometry'] < 0.6: continue
        bad = (side in ('low', 'both') and x < b['p10'] - 1e-9) or (side in ('high', 'both') and x > b['p90'] + 1e-9)
        if bad and level == 'FAIL': f += 1; which.append(key)
        elif bad: w += 1
    res[k] = (f, w, which)
real = [r for k, r in res.items() if d[k]['group'] != 'ours']
c = collections.Counter(r[0] for r in real)
print('real creatures by FAIL lines broken:', dict(sorted(c.items())), 'of', len(real))
for k, (f, w, which) in sorted(res.items(), key=lambda kv: (d[kv[0]]['group'] != 'ours', -kv[1][0])):
    if d[k]['group'] == 'ours' or f >= 2: print(f"  {d[k]['name']:26s} {d[k]['group']:6s} FAIL {f} WARN {w}  {', '.join(which)}")
