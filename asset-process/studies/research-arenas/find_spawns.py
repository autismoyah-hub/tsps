"""find_spawns.py REGEX: boss spawn rows in the server copy's npc-spawns.json, and coordinate-looking lines of its boss plugins."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, json, re, pathlib
S = pathlib.Path(_AP.fix('<TSPS_CHECKOUT>/server'))
rx = re.compile(sys.argv[1], re.I)
rows = json.loads((S / 'data/definitions/npc-spawns.json').read_text(encoding='utf-8'))
for r in rows:
    if rx.search(r.get('name', '')):
        print('spawn', r)
if len(sys.argv) > 2:
    for f in sorted((S / 'plugins').rglob('*')):
        if f.is_file() and f.suffix in ('.js', '.json') and re.search(sys.argv[2], f.as_posix(), re.I):
            for i, l in enumerate(f.read_text(encoding='utf-8', errors='replace').splitlines()):
                if re.search(r'\b\d{4}\b\s*,\s*(y\s*:\s*)?\b\d{4,5}\b|\bx\s*:\s*\d{4}', l) and len(l) < 220:
                    print(f.relative_to(S).as_posix(), i + 1, l.strip()[:200])
