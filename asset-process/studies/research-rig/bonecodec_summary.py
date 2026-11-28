"""What an export folder holds, read back through bonecodec: usage: bonecodec_summary.py EXPORTDIR"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, pathlib, collections
sys.path.insert(0, _AP.fix(r'<ROOT>\studies\research-bosses'))
import bonecodec as B
d = pathlib.Path(sys.argv[1])
for p in sorted(d.glob('*')):
    x = p.read_bytes()
    if p.name.startswith('part-') and p.suffix == '.bin':
        m = B.decode_model(x); w = m['bone_weights'] or []
        print(f"{p.name}: {len(x)} bytes, layout {m['format']}, {len(m['vertices'])} vertices, {len(m['faces'])} faces, influences per vertex {dict(sorted(collections.Counter(len(q) for q in w).items()))}, bones used {len({b for q in w for b, _ in q})}, sums not 255: {sum(1 for q in w if sum(v for _, v in q) != 255)}")
    elif p.name.startswith('base_'):
        b = B.decode_base(x); print(f"{p.name}: {len(x)} bytes, {len(b['types'])} classic groups, {len(b['bones'])} bones, parents {[q['parent'] for q in b['bones']]}")
    elif p.name.startswith('anim_'):
        a = B.decode_anim(x); print(f"{p.name}: {len(x)} bytes, base {a['base']}, range {a['range']}, {len(a['channels'])} channels, {sum(len(c['curve']['keys']) for c in a['channels'])} keys")
