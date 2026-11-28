"""probe.py: what the source cache holds, for choosing free ids (read only)."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, pathlib
sys.path.insert(0, _AP.fix(r'<ROOT>\studies\research-animation'))
import animlib as A

c = A.Cache()
for arch in (0, 1, 2, 5, 7, 22):
    ref = c.reference(arch)
    ids = sorted(ref['groups'])
    raw = c.raw(255, arch)
    span = A.container_span(raw)
    print(f'archive {arch}: protocol {ref["protocol"]} flags {ref["flags"]} version {ref["version"]} groups {len(ids)} max id {ids[-1]} '
          f'ref trailer {len(raw) - span} bytes')
    g = ids[len(ids) // 2]
    r = c.raw(arch, g); span = A.container_span(r)
    print(f'   sample group {g}: stored {len(r)} bytes, trailer {len(r) - span} bytes = {r[span:].hex()}, table version {ref["groups"][g]["version"]}, kind {r[0]}')
ref2 = c.reference(2)
for gid, what in ((6, 'objects'), (9, 'npcs'), (10, 'items'), (12, 'sequences'), (1, 'underlays'), (4, 'overlays')):
    f = ref2['groups'][gid]['files']
    print(f'config group {gid} ({what}): {len(f)} files, max id {max(f)}; version {ref2["groups"][gid]["version"]}')
ref7 = c.reference(7)
ids = sorted(ref7['groups'])
print('models above 65535:', sum(1 for i in ids if i > 65535), 'first', next((i for i in ids if i > 65535), None))
ref5 = c.reference(5)
ids = sorted(ref5['groups'])
print('archive 5 groups sample:', ids[:10], '...', ids[-5:])
g = ref5['groups'][ids[100]]
print('   group', ids[100], {k: (v if k != 'file_names' else v) for k, v in g.items()})
import os
for p in sorted(c.root.iterdir()):
    print(p.name, os.path.getsize(p))
