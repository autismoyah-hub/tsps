"""tab.py KEY[,KEY..] [--md]: the numbers per helm for the named measures (data/measures.json)."""
import sys, json, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import h2
D = json.load(open(h2.HERE / 'data' / 'measures.json'))
keys = sys.argv[1].split(',')
md = '--md' in sys.argv
order = [k for k in h2.ORDER if k in D] + [k for k in D if k not in h2.ORDER]
if md:
    print('| helm | group | ' + ' | '.join(keys) + ' |'); print('|---|---|' + '---:|' * len(keys))
    for k in order: print(f'| {k} | {D[k]["group"]} | ' + ' | '.join(str(D[k].get(q)) for q in keys) + ' |')
else:
    print(f'{"":12}{"":5}' + ''.join(f'{q[-16:]:>17}' for q in keys))
    for k in order: print(f'{k:12}{D[k]["group"]:5}' + ''.join(f'{str(D[k].get(q)):>17}' for q in keys))
