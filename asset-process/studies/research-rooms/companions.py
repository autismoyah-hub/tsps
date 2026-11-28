"""How often a piece of one function touches (8-neighbour footprint contact) a piece of another, in the study set.

usage: companions.py       writes companions.json: for each function X with 5 or more pieces, the share of X pieces
                           touching at least one piece of each function Y, and touching nothing; and the share of X
                           pieces standing against a wall.
"""
import sys, json, collections, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import roommeasure as M

HERE = pathlib.Path(__file__).resolve().parent
study = json.loads((HERE / 'study.json').read_text())
touch = collections.defaultdict(collections.Counter); total = collections.Counter(); alone = collections.Counter(); wall = collections.Counter()
for r in study:
    n = r['norm']; S = {tuple(t) for t in n['tiles']}
    bset = set(M.boundary(list(S)))
    st = [o for o in n['objects'] if o['cls'] in ('furniture', 'light')]
    for i, a in enumerate(st):
        total[a['func']] += 1
        ys = {b['func'] for j, b in enumerate(st) if j != i and M.gap(a, b) == 0}
        if not ys: alone[a['func']] += 1
        for y in ys: touch[a['func']][y] += 1
        if any((x, y, d) in bset for x, y in M.footprint(a) for d in range(4)): wall[a['func']] += 1
out = {}
for x, n in total.most_common():
    if n < 5: continue
    out[x] = dict(n=n, alone=round(alone[x] / n, 2), against_wall=round(wall[x] / n, 2),
                  touches={y: round(c / n, 2) for y, c in touch[x].most_common(6)})
(HERE / 'companions.json').write_text(json.dumps(out, indent=1))
for x, v in out.items(): print(x, v)
