"""census_sprites.py: every visible sprite component in every drawn 239 group (held-out excluded): uses, groups,
commonest sizes, tiling, transparency, flips. Writes recipes/out/sprites.json; prints the top N."""
import sys, json, collections
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
import common as K

N = int(sys.argv[1]) if len(sys.argv) > 1 else 150
names = K.sprite_names()
S = collections.defaultdict(lambda: dict(n=0, groups=set(), size=collections.Counter(), tile=0, trans=collections.Counter(),
                                         flips=collections.Counter()))
for g, rows, by in K.all_states():
    for r in rows:
        if r['kind'] != 5 or not r['visible'] or r['sprites'][0] < 0:
            continue
        e = S[r['sprites'][0]]
        e['n'] += 1; e['groups'].add(g); e['size'][tuple(r['size'])] += 1; e['tile'] += bool(r['sprite_tiling'])
        e['trans'][r['transparency']] += 1; e['flips'][(r['flip_h'], r['flip_v'])] += 1
out = {}
for s, e in sorted(S.items(), key=lambda kv: -len(kv[1]['groups'])):
    out[s] = dict(name=names.get(s, ''), n=e['n'], groups=len(e['groups']), some_groups=sorted(e['groups'])[:12],
                  sizes=[[list(k), v] for k, v in e['size'].most_common(4)], tiled=e['tile'],
                  trans=[[k, v] for k, v in e['trans'].most_common(5)],
                  flips=[[list(k), v] for k, v in e['flips'].most_common(4)])
od = K.HERE / 'recipes' / 'out'; od.mkdir(exist_ok=True)
(od / 'sprites.json').write_text(json.dumps(out, indent=0))
for s, e in list(out.items())[:N]:
    print(s, e['name'], e['n'], e['groups'], e['sizes'][:2], 'tiled', e['tiled'], 'tr', e['trans'][:3], 'fl', e['flips'][:2],
          e['some_groups'][:6])
