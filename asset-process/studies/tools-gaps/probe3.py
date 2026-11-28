import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import json, pathlib, collections
A = pathlib.Path(_AP.fix(r'<REFERENCE_ARCHIVE>\objects'))
for d in sorted(A.glob('shape-*')):
    c = collections.Counter(); ex = {}
    n = 0
    for p in d.glob('*.json'):
        j = json.loads(p.read_text(encoding='utf-8')); n += 1
        ms = j.get('model_shapes')
        k = 'none' if not ms else ('same-as-folder-only' if set(ms) == {int(d.name[6:])} else ('mixed-with-folder' if int(d.name[6:]) in ms else 'mixed-without-folder'))
        k += ' / n_models' + ('=' if len(j['model_ids']) == len(ms or j['model_ids']) else '!=') + 'n_shapes'
        c[k] += 1; ex.setdefault(k, (j['id'], j['model_ids'], ms))
    print(d.name, n, dict(c)); print('   ', ex)
ids = collections.Counter(int(p.stem) for p in A.glob('shape-*/*.json'))
print('ids in more than one folder:', sum(1 for v in ids.values() if v > 1), [k for k, v in ids.items() if v > 1][:10])
for i in (20624, 13686, 22472, 13731):
    p = next(A.glob(f'shape-*/{i}.json')); j = json.loads(p.read_text(encoding='utf-8'))
    print(p.parent.name, {k: j[k] for k in ('id', 'name', 'model_ids', 'model_shapes', 'vertices', 'faces', 'bounds_min', 'bounds_max', 'offset', 'recolors') if k in j})
