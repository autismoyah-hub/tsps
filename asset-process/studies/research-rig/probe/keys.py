import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, collections
sys.path.insert(0, _AP.fix(r'<ROOT>\studies\research-bosses'))
import bonelib as L, bonecodec as B
sid = int(sys.argv[1]); aid, rng = L.anim_id(sid); a = B.decode_anim(L.anim_bytes(aid))
seen = collections.Counter(); shown = set()
for c in a['channels']:
    ks = c['curve']['keys']; pat = (c['kind'], c['channel'] if c['kind'] != 1 else (c['channel'] - 1) // 3, len(ks), c['curve']['byte0'])
    seen[pat] += 1
    if pat not in shown and len(shown) < int(sys.argv[2]) if len(sys.argv) > 2 else 8:
        shown.add(pat); print(pat, 'target', c['target'], 'ch', c['channel'], {k: v for k, v in c['curve'].items() if k != 'keys'})
        for k in ks[:5] + ks[-3:]: print('   ', k)
print(seen.most_common(20))
