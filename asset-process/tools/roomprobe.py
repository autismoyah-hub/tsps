"""World-space probe of the E114 room: size of whatever stands near a tile position."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import osrscache as o
A = pathlib.Path(_AP.fix(r'<E114_SCENE>'))
place = {}
for line in (A / 'floor.bwr').read_text().splitlines():
    p = line.split()
    if p[0] == 'O': place[int(p[1])] = (int(p[2]), int(p[3]))
world = []
for oid, (tx, tz) in place.items():
    m = o.decode_model((A / 'package' / f'm{oid-64000}.bin').read_bytes())
    for (x, y, z) in m['vertices']:
        world.append((tx + 0.5 + x / 128, -y / 128, tz + 0.5 + z / 128))
xs = [w[0] for w in world]; zs = [w[2] for w in world]; ys = [w[1] for w in world]
print('room extent tiles x %.2f..%.2f  z %.2f..%.2f  height %.2f..%.2f' % (min(xs), max(xs), min(zs), max(zs), min(ys), max(ys)))
for arg in sys.argv[1:]:
    cx, cz, r = map(float, arg.split(','))
    sel = [w for w in world if abs(w[0] - cx) <= r and abs(w[2] - cz) <= r and w[1] > 0.02]
    if not sel: print(arg, 'nothing'); continue
    print(arg, 'x %.2f..%.2f z %.2f..%.2f top %.2f tiles  = %d x %d x %d units' % (
        min(w[0] for w in sel), max(w[0] for w in sel), min(w[2] for w in sel), max(w[2] for w in sel), max(w[1] for w in sel),
        (max(w[0] for w in sel) - min(w[0] for w in sel)) * 128, max(w[1] for w in sel) * 128, (max(w[2] for w in sel) - min(w[2] for w in sel)) * 128))
