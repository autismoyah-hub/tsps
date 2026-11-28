import sys, json, pathlib, random, statistics as st
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import osrscache as o, measure as m
d = m.ARCH / 'objects' / 'shape-10'
ids = sorted(int(p.stem) for p in d.glob('*.json'))
random.seed(1)
print('band        n  faces  smooth  L50  L90  S50  S90  n90  medarea')
for lo in range(0, 64000, 4000):
    band = [i for i in ids if lo <= i < lo + 4000]
    random.shuffle(band)
    rows, seen = [], set()
    for i in band:
        j = json.loads((d / f'{i}.json').read_text())
        key = tuple(j.get('model_ids') or [])
        if not key or key in seen or (j.get('faces') or 0) < 100: continue
        seen.add(key)
        try: rows.append(m.stats(o.merge([o.model(k) for k in key])))
        except Exception: continue
        if len(rows) >= 80: break
    if not rows: continue
    md = lambda k: st.median(r[k] for r in rows if r[k] is not None)
    print(f'{lo:5d}-{lo+3999:5d} {len(rows):3d} {md("faces"):6.0f} {md("smooth"):6.2f} {md("L50"):5.0f} {md("L90"):4.0f} {md("S50"):4.0f} {md("S90"):4.0f} {md("n90"):4.0f} {md("medarea"):7.0f}')
