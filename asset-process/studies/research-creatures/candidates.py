"""List candidate creatures by model-id band, for choosing the era groups by hand.

usage: candidates.py LO HI [--minfaces N] [--combat N]
Distinct model lists whose model ids ALL lie in LO..HI, not on the player skeleton 0, one line per distinct name
(the first definition of that name), sorted by lowest model id.
"""
import sys, pathlib; sys.path.insert(0, str(pathlib.Path(__file__).parent))
import find
a = sys.argv; lo, hi = int(a[1]), int(a[2])
minf = int(a[a.index('--minfaces') + 1]) if '--minfaces' in a else 150
minc = int(a[a.index('--combat') + 1]) if '--combat' in a else 0
seen = set(); rows = []
for n in find.npcs:
    ms = [m for m in n['models'] if m in find.models]
    if not ms or n['name'] in seen or n['name'] == 'null': continue
    if not all(lo <= m <= hi for m in ms): continue
    if find.skel(n)[0] in (0, -1): continue
    if sum(find.models[m]['faces'] for m in ms) < minf or n['combat'] < minc: continue
    seen.add(n['name']); rows.append((min(ms), n))
for _, n in sorted(rows, key=lambda r: r[0]):
    print(find.line(n).replace('weights False ', '').replace(' classic', ''))
