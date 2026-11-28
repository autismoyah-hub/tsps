import sys,pathlib,json,collections
sys.path.insert(0,str(pathlib.Path(__file__).parent))
from models import *
from materials import metadata
read=lambda f:json.loads((ROOT/f).read_text())
for rev,d in read('census.json').items():
 print(rev,d['versions'],d['triangle_opcodes'],d['vertex_flags'],d['texture_types'],d['render_values'],d['attachment_models'],{k:v[0] for k,v in d['examples'].items()},d.get('config_groups'),d.get('texture_sizes'),{k:d['indexes'].get(k) for k in ['0','1','7','9']})
proof=read('pixel-proof.json')
print('proof',len(proof),sum(sum(r['foreground']) for r in proof),min(min(r['foreground']) for r in proof))
for r in read('study.json'):
 if r['kind']=='item':continue
 print(r['name'],r['id'],r['models'],r['hd'],r['ref']['name'],r['ref']['id'],r['ref']['metrics'],r['ref'].get('anim',{}))
c=Cache(614)
for i in [59609,51518,55250,56815,55402,56817,56816,56819,48292,55399,42715]:
 m=decode(c.group(7,i));print(i,collections.Counter(m['texture_types']),m['hd'].get('attachments'))
c=Cache(242)
for i in [68,53744,53387,53362,50524]:
 print('242',i,[(k,len(m[k])) for m in [osrs.decode_model(c.group(7,i))] for k in ('vertices','faces')])

