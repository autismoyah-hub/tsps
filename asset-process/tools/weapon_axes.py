"""Real weapons measured along their own axes (ground models lie diagonally, so bounds mislead)."""
import sys, json, pathlib, statistics as st
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import numpy as np, osrscache as o, measure as m
rows = []
for j in m.newest('items', 'slot-3', 120, lambda j: j.get('inventory_model', 0) > 0):
    try: md = o.model(j['inventory_model'])
    except Exception: continue
    v = np.array(md['vertices'], float); v -= v.mean(0)
    w, vec = np.linalg.eigh(np.cov(v.T))
    p = v @ vec[:, ::-1]                       # longest axis first
    e = p.max(0) - p.min(0)
    rows.append((j['name'], len(md['faces']), e[0], e[1], e[2]))
q = lambda xs, f: sorted(xs)[int(f * (len(xs) - 1))]
L = [r[2] for r in rows]; W = [r[3] / r[2] for r in rows]; T = [r[4] / r[2] for r in rows]; TA = [r[4] for r in rows]
print('real weapons, ground model, n', len(rows))
for name, xs in (('length (units)', L), ('width / length', W), ('thickness / length', T), ('thickness (units)', TA), ('faces', [r[1] for r in rows])):
    print(f'  {name:20s} p10 {q(xs,.1):7.2f}  median {q(xs,.5):7.2f}  p90 {q(xs,.9):7.2f}')
for r in rows[:14]: print('   ', r[0][:26].ljust(26), 'faces', r[1], 'len %.0f wid %.0f thick %.0f' % r[2:])
