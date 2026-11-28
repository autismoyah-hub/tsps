"""List chosen measures for every creature, sorted by the first. usage: q.py KEY[,KEY..]"""
import sys, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import measure_paint as mp
keys = sys.argv[1].split(',')
d = mp.load()
print(f"{'':26s} {'':6s} " + ' '.join(f'{k[:13]:>13s}' for k in keys))
f = lambda x: f'{x:13.3f}' if isinstance(x, (int, float)) else f'{str(x)[:13]:>13s}'
for k, v in sorted(d.items(), key=lambda kv: (kv[1]['m'].get(keys[0]) is None, kv[1]['m'].get(keys[0]) or 0)):
    print(f"{v['name']:26s} {v['group']:6s} " + ' '.join(f(v['m'].get(q)) for q in keys))
