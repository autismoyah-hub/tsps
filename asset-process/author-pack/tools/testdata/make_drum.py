"""Test model for check_predict.py: an eight-sided drum, lower half flat-marked, upper half smooth, flat-marked lid.
Large faces in eight directions, so every facing of both shading kinds can be sampled in a line-up.
usage: make_drum.py OUT.bin"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys
sys.path.insert(0, _AP.fix(r'<ROOT>\tools'))
import author

m = author.Mesh()
m.out = lambda c: (c[0], 0, c[2])
r0, r1, r2 = m.ring(0, 50, 8, 0.3), m.ring(50, 50, 8, 0.3), m.ring(100, 50, 8, 0.3)
for i in range(8):
    j = (i + 1) % 8
    m.quad(r0[i], r0[j], r1[j], r1[i], author.hsl(6, 4, 60), flat=True)
    m.quad(r1[i], r1[j], r2[j], r2[i], author.hsl(20, 4, 60))
m.out = lambda c: (0, 1, 0)
top = m.vert(0, 100, 0)
for i in range(8):
    m.tri(r2[i], r2[(i + 1) % 8], top, author.hsl(40, 3, 70), flat=True)
open(sys.argv[1], 'wb').write(m.encode())
print(sys.argv[1], len(m.v), 'vertices', len(m.f), 'faces')
