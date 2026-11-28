"""Check that the repaint kept the watcher's shape: vertices, labels, original faces, bounds, outline.
usage: verify_repaint.py"""
import sys, pathlib
import numpy as np
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import paintlib as pl, meshio
from repaint_watcher import as_cmesh
a = meshio.load(HERE.parent / 'creature-stone-watcher-2' / 'creature-stone-watcher-2.bin'); b = meshio.load(HERE / 'repaint' / 'watcher-2-repaint.bin')
na, nb = len(a['vertices']), len(b['vertices'])
print('vertices', na, '->', nb, '| first', na, 'identical:', a['vertices'] == b['vertices'][:na], '| labels identical:', a['vertex_skins'] == b['vertex_skins'][:na])
print('faces', len(a['faces']), '->', len(b['faces']), '| faces unchanged in place:', sum(1 for x, y in zip(a['faces'], b['faces']) if x == y), 'of', len(a['faces']))
print('new vertex labels all among the old:', set(b['vertex_skins'][na:]) <= set(a['vertex_skins']))
for k in ('alpha', 'render_types', 'depth', 'priorities'): print(k, 'same on unchanged faces:', all(p == q for p, q, x, y in zip(a[k], b[k], a['faces'], b['faces']) if x == y))
A, Bm = np.array(a['vertices']), np.array(b['vertices'])
print('bounds', A.min(0).tolist(), A.max(0).tolist(), '->', Bm.min(0).tolist(), Bm.max(0).tolist())
sa, sb = pl.skin('a', mesh=as_cmesh(a)), pl.skin('b', mesh=as_cmesh(b))
print('skin area', round(sa['tot']), '->', round(sb['tot']))
for yaw in (0, 90, 180, 25):
    ia = pl.paint_view(sa, yaw, 12, ppu=1.0, return_ids=True)[1] >= 0; ib = pl.paint_view(sb, yaw, 12, ppu=1.0, return_ids=True)[1] >= 0
    print(f'outline at yaw {yaw}: pixels {ia.sum()} -> {ib.sum()}, differing {(ia != ib).sum() if ia.shape == ib.shape else "shape differs"}')
