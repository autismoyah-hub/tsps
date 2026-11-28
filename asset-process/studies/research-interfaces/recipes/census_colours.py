"""census_colours.py: colours of visible texts (base and <col> spans), rectangles and lines, by number of groups
(held-out excluded). Writes recipes/out/colours.json: the source of the named palette in INTERFACES.md."""
import sys, json, re, collections
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
import common as K

G = collections.defaultdict(lambda: collections.defaultdict(set))
for g, rows, by in K.all_states():
    for r in rows:
        if not r['visible']:
            continue
        if r['kind'] == 4 and r['text'] and r['font'] >= 0:
            G[r['colors'][0] & 0xffffff]['text'].add(g)
            for m in re.finditer(r'<col=([0-9a-fA-F]{6})>', r['text']):
                G[int(m.group(1), 16)]['span'].add(g)
        elif r['kind'] == 3 and r['transparency'] < 255:
            G[r['colors'][0] & 0xffffff]['fill' if r['filled'] else 'outline'].add(g)
        elif r['kind'] == 9:
            G[r['colors'][0] & 0xffffff]['line'].add(g)
out = []
for c, d in G.items():
    allg = set().union(*d.values())
    out.append(dict(colour=f'{c:06x}', groups=len(allg), **{k: len(v) for k, v in d.items()}))
out.sort(key=lambda e: -e['groups'])
(K.HERE / 'recipes' / 'out' / 'colours.json').write_text(json.dumps(out, indent=0))
print(len(out), 'colours;', sum(1 for e in out if e['groups'] >= 3), 'in >= 3 groups')
for e in out[:90]:
    print(e)
