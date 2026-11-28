"""Style card for real armour, from the male WORN model: counts, colours, and how vertices are labelled for animation."""
import sys, json, pathlib, collections
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import osrscache as o, measure as m
from measure import area
SLOTS = {'slot-0': 'head', 'slot-4': 'body', 'slot-7': 'legs', 'slot-10': 'feet', 'slot-9': 'hands', 'slot-1': 'cape', 'slot-5': 'shield'}
for sub, nm in SLOTS.items():
    d = m.ARCH / 'items' / sub
    if not d.exists(): continue
    rows = []; seen = set(); labels = collections.Counter(); per = []
    for i in sorted((int(p.stem) for p in d.glob('*.json')), reverse=True):
        j = json.loads((d / f'{i}.json').read_text())
        ids = tuple(k for k in (j.get('wear_models') or [[-1]])[0] if k not in (-1, None))
        if not ids or ids in seen: continue
        seen.add(ids)
        try: mods = [o.model(k) for k in ids]
        except Exception: continue
        nf = sum(len(x['faces']) for x in mods)
        vl = collections.Counter(l for x in mods for l in (x['vertex_skins'] or [0] * len(x['vertices'])))
        cols = collections.Counter(c for x in mods for c in x['colors'])
        ys = [-p[1] for x in mods for p in x['vertices']]
        rows.append(dict(id=i, name=j['name'], models=len(ids), faces=nf, nlabel=len(vl), ncol=len(cols), lo=min(ys), hi=max(ys),
                         flat=sum(1 for x in mods for r in x['render_types'] if r == 1) / nf, nprio=len(set(p for x in mods for p in x['priorities']))))
        labels.update(vl.keys()); per.append(vl)
        if len(rows) >= 60: break
    if not rows: continue
    q = lambda k, f: sorted(r[k] for r in rows)[int(f * (len(rows) - 1))]
    print(f'== {nm} ({sub}), newest {len(rows)} distinct')
    for k in ('faces', 'models', 'nlabel', 'ncol', 'lo', 'hi', 'nprio'):
        print(f'   {k:7s} p10 {q(k,.1):6.1f}  median {q(k,.5):6.1f}  p90 {q(k,.9):6.1f}')
    print('   labels used by at least a third of items:', sorted(l for l, n in labels.items() if n >= len(rows) / 3))
    for r in rows[:4]: print('     ', r['id'], r['name'][:28], 'faces', r['faces'], 'labels', r['nlabel'], 'colours', r['ncol'], 'height', r['lo'], 'to', r['hi'])
