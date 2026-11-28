import sys,pathlib,json
sys.path.insert(0,str(pathlib.Path(__file__).parent))
from definitions import *
c=Cache(561);lookup={i:[] for i,j,n,f in json.loads((ROOT/'later-matches.json').read_text())}
for k in ('npc','object','item','spot'):
 for i,data in records(c,k):
    d=parse(k,data,561)
    for mid in d['models']:
        if mid in lookup and len(lookup[mid])<8:lookup[mid].append([k,i,d['name']])
print(lookup)
(ROOT/'later-identities.json').write_text(json.dumps(lookup,indent=1))

