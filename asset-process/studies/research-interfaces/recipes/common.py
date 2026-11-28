"""common.py: shared loading for the v2 recipe census (Teacher side).

Every census runs over state/<group>.jsonl (the tooling client's drawn state of each 239 group after its onLoad
scripts), EXCLUDING the six held-out panels of recipes/heldout.json (never used to tune or count a recipe)."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, json, re, collections
from pathlib import Path

HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))

HELDOUT = set(c['group'] for c in json.loads((HERE / 'recipes' / 'heldout.json').read_text())['chosen'])
GAMEVAL = Path(_AP.fix(r'<TSPS_CHECKOUT>/client/runelite/api/gameval'))


def sprite_names():
    p = GAMEVAL / 'SpriteID.ts'
    if not p.exists():
        return {}
    return {int(i): n for n, i in re.findall(r'export const (\w+) = (\d+);', p.read_text())}


def groups():
    out = []
    for p in sorted((HERE / 'state').glob('*.jsonl'), key=lambda p: int(p.stem) if p.stem.isdigit() else 10 ** 9):
        if not p.stem.isdigit():
            continue
        g = int(p.stem)
        if g in HELDOUT:
            continue
        out.append(g)
    return out


def load(g):
    rows = [json.loads(l) for l in (HERE / 'state' / f'{g}.jsonl').read_text().splitlines() if l.strip()]
    by = {r['id']: r for r in rows}
    order = [r['id'] for r in rows]
    # visible = not hidden and no hidden ancestor
    vis = {}
    for r in rows:
        p = r['parent']
        vis[r['id']] = (not r['hidden']) and (p < 0 or vis.get(p, True))
    for r in rows:
        r['visible'] = vis[r['id']]
        r['order'] = order.index(r['id']) if False else None
    for n, i in enumerate(order):
        by[i]['order'] = n
    return rows, by


_defs = {}
_c239 = None


def defs(g):
    """{component id: definition} of group g in the 239 cache (listeners: the onLoad script that builds a frame)."""
    global _c239
    if g not in _defs:
        import ifcache as C
        if _c239 is None:
            _c239 = C.Cache()
        try:
            _defs[g] = {w['id']: w for w in C.load_interface(_c239, g)}
        except Exception:
            _defs[g] = {}
    return _defs[g]


def host_script(g, by, r):
    """The onLoad script id of the nearest cache-defined ancestor-or-self of state row r (who built it)."""
    D = defs(g)
    i = r['id']
    while i in by:
        row = by[i]
        if row.get('child_index', -1) < 0 and i in D:
            ol = D[i]['listeners'].get('onLoad')
            if ol:
                return ol[0]
            if row is not r:
                return None
        i = row['parent']
    return None


def all_states():
    for g in groups():
        yield g, *load(g)
