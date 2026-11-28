"""Style card for real weapons, from the WORN model (what is seen in a figure's hand), along the weapon's own axes."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, json, pathlib, collections, re
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import numpy as np, osrscache as o, measure as m
from measure import area
WANT = re.compile(r"halberd|fang|axe|recurve|shadow|voidwaker|crossbow|sunspear|khopesh|claws|staff|tecpatl|barrage|sceptre|scimitar|sword|mace|maul|spear|bow$|trident|dagger|whip|rapier|blade|hammer|bludgeon|scythe|wand|ballista|flail|sickle", re.I)
SKIP = re.compile(r"trophy|scroll|crate|banner|dummy|net|\(o\)|\(u\)|uncharged|\(deadman\)|broken|\(l\)", re.I)
d = m.ARCH / 'items' / 'slot-3'
rows = []; seen = set()
for i in sorted((int(p.stem) for p in d.glob('*.json')), reverse=True):
    j = json.loads((d / f'{i}.json').read_text())
    n = j.get('name') or ''
    wm = (j.get('wear_models') or [[-1]])[0][0]
    if wm in (-1, None) or wm in seen or not WANT.search(n) or SKIP.search(n): continue
    seen.add(wm)
    try: md = o.model(wm)
    except Exception: continue
    v = np.array(md['vertices'], float); c = v - v.mean(0)
    w, vec = np.linalg.eigh(np.cov(c.T)); p = c @ vec[:, ::-1]; e = p.max(0) - p.min(0)
    ar = collections.Counter()
    for f, col in zip(md['faces'], md['colors']): ar[col] += area(md['vertices'], f)
    tot = sum(ar.values()) or 1; acc = 0; n90 = 0
    for _, a in ar.most_common():
        acc += a; n90 += 1
        if acc >= .9 * tot: break
    L = sorted((o.hsl_split(col)[2], a) for col, a in ar.items()); acc = 0; med = 0
    for l, a in L:
        acc += a
        if acc >= tot / 2: med = l; break
    rows.append(dict(id=i, name=n, model=wm, faces=len(md['faces']), length=e[0], width=e[1], thick=e[2], ncol=len(ar), n90=n90, L50=med,
                     flat=sum(1 for r in md['render_types'] if r == 1) / len(md['faces']), nprio=len(set(md['priorities'])),
                     maxsat=max(o.hsl_split(col)[1] for col in ar)))
    if len(rows) >= 60: break
q = lambda k, f: sorted(r[k] for r in rows)[int(f * (len(rows) - 1))]
print('real weapons, worn model, newest', len(rows))
for k in ('faces', 'length', 'width', 'thick', 'ncol', 'n90', 'L50', 'flat', 'nprio'):
    print(f'  {k:7s} p10 {q(k,.1):7.2f}  median {q(k,.5):7.2f}  p90 {q(k,.9):7.2f}')
print('  thickness/length median %.3f   width/length median %.3f' % (np.median([r['thick']/r['length'] for r in rows]), np.median([r['width']/r['length'] for r in rows])))
for r in rows[:24]: print(f"   {r['id']} {r['name'][:26]:26s} model {r['model']:6d} f{r['faces']:4d}  {r['length']:.0f} x {r['width']:.0f} x {r['thick']:.0f}  colours {r['ncol']} ({r['n90']} for 90%)  L50 {r['L50']}")
json.dump(rows, open(_AP.fix(r'<ROOT>\measure\weapon-card.json'), 'w'), indent=1, default=float)
