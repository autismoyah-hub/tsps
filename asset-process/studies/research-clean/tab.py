"""print chosen measures per creature from data/explore*.json: tab.py key,key,.. [--sort key]"""
import sys, json, pathlib
HERE = pathlib.Path(__file__).resolve().parent
rows = {}
for f in ('explore.json', 'explore2.json', 'explore3.json'):
    p = HERE / 'data' / f
    if p.exists():
        for k, r in json.loads(p.read_text()).items(): rows.setdefault(k, {}).update(r)
keys = sys.argv[1].split(',')
srt = sys.argv[sys.argv.index('--sort') + 1] if '--sort' in sys.argv else keys[0]
print(f"{'creature':22s} st " + ' '.join(f'{k[:13]:>13s}' for k in keys))
for k, r in sorted(rows.items(), key=lambda kv: kv[1].get(srt, 0)):
    print(f"{r['name'][:22]:22s} {r['style']}  " + ' '.join(f'{r.get(q, float("nan")):13.3f}' for q in keys))
