import json,collections,sys
from pathlib import Path
HERE=Path(__file__).resolve().parent
sys.path.insert(0,str(HERE.parents[1]/'tools'))
import osrscache as o
for row in json.loads((HERE/'proof.json').read_bytes()):
    if 'error' in row:print(row);continue
    p=HERE/'proof'/f"{row['kind']}-{row['id']}"/'mesh.bin'
    m=o.decode_model(p.read_bytes())
    print(row['id'],row['different'],sum(t>=0 and bool(a) for t,a in zip(m['face_textures'],m['alpha'])), 'mapped',sum(c>=0 for c in m['texture_coords']))
r=[json.loads(s) for s in (HERE/'blend-regression.jsonl').read_text().splitlines()]
print('blend sources',sum(len(s['paths']) for s in r),'distinct',len(r),'success',sum(1 for s in r if 'error' not in s),'trials',sum(len(s['trials']) for s in r),'different',sum(s['trials'].count(False) for s in r),'matched stored',len({p for s in r for p in s['matches']}),'failures',[s['paths'][0] for s in r if 'error' in s])
rr={r['hash']:r for r in map(json.loads,(HERE/'binary-regression.jsonl').read_text().splitlines())}
print('binary exact',sum(s['count'] for s in rr.values() if s.get('exact')),len(rr),'outstanding',[s for s in rr.values() if not s.get('exact')][:10])
