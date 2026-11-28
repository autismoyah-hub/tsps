"""census.py: the stand-off of EVERY real head-slot item with a distinct male worn mesh (a falsification check of the
fit lines: is any real helm that hides the hair as far off the head as ours?). Writes census.json."""
import sys, pathlib, json
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
import lib as L, fitlib as F

seen = {}; rows = []
for i, e in sorted(L.idx().items()):
    if e['wp'][0] != 0: continue
    wm = tuple(k for k in e['wm'][0] if k not in (-1, None))
    if not wm or wm in seen: continue
    seen[wm] = i
    try:
        m = L.mesh_real(i)
        s = F.standoff(m)
    except Exception as ex:
        continue
    V = m['V']
    rows.append(dict(id=i, name=e['name'], hides_hair=8 in e['wp'][1:], hides_jaw=11 in e['wp'][1:], faces=len(m['F']),
                     **{k: v for k, v in s.items() if not k.startswith('_')}))
json.dump(rows, open(pathlib.Path(__file__).parent / 'census.json', 'w'), indent=1)
hh = [r for r in rows if r['hides_hair'] and r['cover'] >= 0.8 and 'so_med' in r]
print(len(rows), 'distinct head meshes;', len(hh), 'hide the hair and cover 0.8 or more of the head')
for key in ('so_p10', 'so_med', 'so_brow', 'so_temple', 'so_cheek', 'so_crown'):
    v = np.array([r[key] for r in hh if r.get(key) is not None])
    print(f'{key:10} n {len(v):4}  pct 5/25/50/75/95: ' + ' '.join(f'{np.percentile(v, q):5.1f}' for q in (5, 25, 50, 75, 95)) + f'   max {v.max():.1f}')
for key, lim in (('so_p10', 2.5), ('so_med', 4.1)):
    over = sorted([r for r in hh if r.get(key) is not None and r[key] >= lim], key=lambda r: -r[key])
    print(f'{key} >= {lim} (the LEAST of our seven): {len(over)} of {len(hh)}')
    for r in over[:40]: print('   ', r['id'], r['name'], key, r[key], 'med', r['so_med'], 'p10', r['so_p10'], 'faces', r['faces'])
bothv = [r for r in hh if r['so_p10'] >= 2.5 and r['so_med'] >= 4.1]
print('both at once:', len(bothv), [(r['id'], r['name']) for r in bothv])
