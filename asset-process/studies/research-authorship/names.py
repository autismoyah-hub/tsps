"""names.py: grep the cache's gameval name tables (archive 24: 1 NPC, 6 LOC, 7 SEQ, 8 SPOTANIM).
usage: names.py KIND REGEX [REGEX..]     KIND in npc, loc, seq, spot, all
Prints id and name of every match. Read only.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, re, pathlib
sys.dont_write_bytecode = True
sys.path.insert(0, _AP.fix(r'<ROOT>\studies\research-effects'))
import fxlib
K = dict(npc=1, loc=6, seq=7, spot=8)
kind = sys.argv[1]; pats = [re.compile(p, re.I) for p in sys.argv[2:]]
for k in (K if kind == 'all' else [kind]):
    nm = fxlib.names(K[k])
    for i in sorted(nm):
        if any(p.search(nm[i]) for p in pats): print(k, i, nm[i])
