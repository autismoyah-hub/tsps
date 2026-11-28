"""explore_head.py: print the bare head (hair + jaw kits) as numbers: widths and depths by height."""
import sys, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
import lib as L
hv, hf = L.kit('hair'); jv, jf = L.kit('jaw'); bv, bf = L.kit('body')
print('hair bounds', hv.min(0), hv.max(0), len(hf))
print('jaw  bounds', jv.min(0), jv.max(0), len(jf))
print('body bounds', bv.min(0), bv.max(0), len(bf))
allv = np.vstack([hv, jv])
for h in range(196, 160, -2):
    sel = allv[(allv[:, 1] >= h - 1.5) & (allv[:, 1] <= h + 1.5)]
    if len(sel):
        print(h, 'x', sel[:, 0].min(), sel[:, 0].max(), 'z', sel[:, 2].min(), sel[:, 2].max(), 'n', len(sel))
print('jaw verts', jv.tolist())
print('hair colours', sorted(set(map(tuple, L.KITS['hair']['face_colours_hsl']))))
print('jaw colours', sorted(set(map(tuple, L.KITS['jaw']['face_colours_hsl']))))
print('body top verts', sorted([tuple(v) for v in bv if v[1] >= 160], key=lambda v: -v[1])[:30])
for k in (26382, 22326, 12931, 24271, 30750):
    m = L.mesh_real(k); V = m['V']
    print(L.item_name(k), 'wp', L.wear_positions(k), 'bounds', V.min(0), V.max(0), 'faces', len(m['F']), 'prio', sorted(set(m['prio'])))
for r in L.OURS:
    m = L.mesh_ours(r[1]); V = m['V']
    print(r[0], r[2], 'bounds', V.min(0), V.max(0), 'faces', len(m['F']), 'prio', sorted(set(m['prio'])))
