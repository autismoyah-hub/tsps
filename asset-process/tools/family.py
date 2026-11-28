"""Real objects whose painted palette is close to a given real object's. usage: family.py OBJECT_ID [N] [min_faces]"""
import sys, json, pathlib, collections
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import osrscache as o, scene, compare
ref = int(sys.argv[1]); N = int(sys.argv[2]) if len(sys.argv) > 2 else 30; minf = int(sys.argv[3]) if len(sys.argv) > 3 else 150
target = compare.colour_shares(compare.real_model(ref))
d = scene.ARCH / 'objects' / 'shape-10'
out = []; seen = set()
for p in d.glob('*.json'):
    j = json.loads(p.read_text()); k = tuple(j.get('model_ids') or [])
    if not k or k in seen or (j.get('faces') or 0) < minf: continue
    seen.add(k)
    try: sh = compare.colour_shares(compare.real_model(j['id']))
    except Exception: continue
    out.append((compare.colour_distance(sh, target), j['id'], j['name'], j['faces'], [round(b - a) for a, b in zip(j['bounds_min'], j['bounds_max'])]))
out.sort()
for dist, i, name, f, b in out[:N]: print(f'{dist:5.2f} {i:6d} {name[:28]:28s} f{f:5d} {b}')
