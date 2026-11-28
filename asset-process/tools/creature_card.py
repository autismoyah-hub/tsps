"""First facts about real creatures and bosses: size, faces, labels, and which movement mechanism their models carry."""
import sys, json, pathlib, collections, re
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import osrscache as o, measure as m
rows = []
for sub in (m.ARCH / 'npcs').iterdir():
    mt = re.match(r'combat-(\d+)-', sub.name)
    if not mt: continue
    lo = int(mt[1])
    for p in sub.glob('*.json'):
        j = json.loads(p.read_text()); ids = tuple(j.get('model_ids') or [])
        if not ids: continue
        rows.append((lo, j['id'], j['name'], ids, j.get('faces') or 0, j.get('bounds_min'), j.get('bounds_max'), tuple(j.get('animations') or [])))
seen = set(); out = collections.defaultdict(list)
for lo, i, name, ids, faces, bmin, bmax, anims in sorted(rows, key=lambda r: -r[1]):
    if ids in seen: continue
    seen.add(ids)
    band = 'combat 300 and over' if lo >= 300 else 'combat 100 to 299' if lo >= 100 else 'combat under 100'
    if len(out[band]) >= 150: continue
    try: mods = [o.model(k) for k in ids]
    except Exception: continue
    labels = set(l for x in mods for l in (x['vertex_skins'] or []))
    h = (bmax[1] - bmin[1]) if bmin else 0
    w = max(bmax[0] - bmin[0], bmax[2] - bmin[2]) if bmin else 0
    out[band].append(dict(name=name, id=i, faces=sum(len(x['faces']) for x in mods), nmodels=len(ids), nlabels=len(labels),
                          weights=any(x['has_bone_weights'] for x in mods), height=h, width=w))
q = lambda xs, f: sorted(xs)[int(f * (len(xs) - 1))]
for band in ('combat under 100', 'combat 100 to 299', 'combat 300 and over'):
    r = out[band]
    if not r: continue
    print(f'== {band}: newest {len(r)} distinct')
    for k in ('faces', 'nmodels', 'nlabels', 'height', 'width'):
        xs = [x[k] for x in r]; print(f'   {k:8s} p10 {q(xs,.1):7.0f}  median {q(xs,.5):7.0f}  p90 {q(xs,.9):7.0f}')
    print(f'   carry bone weights (the newer mechanism): {100*sum(x["weights"] for x in r)/len(r):.0f}%   built from several models: {100*sum(x["nmodels"]>1 for x in r)/len(r):.0f}%')
    for x in r[:6]: print('     ', x['id'], x['name'][:26], 'faces', x['faces'], 'models', x['nmodels'], 'labels', x['nlabels'], 'weights', x['weights'], 'size', round(x['width']), 'x', round(x['height']))
