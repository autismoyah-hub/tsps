"""Which animations the owner's server (TSPS) plays for an item: stand, walk, run, block, attack.

usage: tsps_anims.py ITEM_ID [ITEM_ID...]   |   tsps_anims.py --name PATTERN
Source: the server's own item data, `data/definitions/item-gameplay.json` in the copy of the TSPS repository on the
asset machine. The client does not hold these; the server sends them. Use them, not remembered numbers, when posing
a figure with `pose.py`. An item with no `attackAnim` takes its attack from the server's weapon profile
(`src/main/typescript/elvarg/game/content/combat/WeaponProfile.ts` in the same copy).
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, json, re, pathlib
DATA = pathlib.Path(_AP.fix(r'<TSPS_CHECKOUT>\server\data\definitions\item-gameplay.json'))
d = json.loads(DATA.read_text(encoding='utf-8'))
items = d if isinstance(d, list) else list(d.values())
keys = ('standAnim', 'walkAnim', 'runAnim', 'blockAnim', 'attackAnim')
if sys.argv[1] == '--name':
    rx = re.compile(sys.argv[2], re.I)
    sel = [e for e in items if e and rx.search(str(e.get('name', '')))][:40]
else:
    want = set(map(int, sys.argv[1:]))
    sel = [e for e in items if e and e.get('id') in want]
for e in sel:
    print(e.get('id'), e.get('name'), {k: e[k] for k in keys if k in e})
