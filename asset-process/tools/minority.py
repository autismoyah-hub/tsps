import sys, json, pathlib, random, collections
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import osrscache as o, measure as m
random.seed(5)
d = m.ARCH / 'objects' / 'shape-10'; ids = [int(p.stem) for p in d.glob('*.json')]; random.shuffle(ids)
n = anyflat = mostflat = anydup = 0; shares = []; seen = set()
for i in ids:
    j = json.loads((d / f'{i}.json').read_text()); k = tuple(j.get('model_ids') or [])
    if not k or k in seen or (j.get('faces') or 0) < 100: continue
    seen.add(k)
    try: md = o.merge([o.model(x) for x in k])
    except Exception: continue
    n += 1
    fl = sum(1 for r in md['render_types'] if r == 1) / len(md['faces'])
    shares.append(fl); anyflat += fl > 0; mostflat += fl > 0.5
    anydup += any(c > 1 for c in collections.Counter(md['vertices']).values())
    if n >= 600: break
s = sorted(shares)
print(f'real objects n={n}: with any flat-shaded face {100*anyflat/n:.0f}%, mostly flat {100*mostflat/n:.0f}%, with any shared position (merged models) {100*anydup/n:.0f}%')
print('flat share of faces: p50 %.2f p75 %.2f p90 %.2f p95 %.2f' % tuple(s[int(q*(n-1))] for q in (.5,.75,.9,.95)))
