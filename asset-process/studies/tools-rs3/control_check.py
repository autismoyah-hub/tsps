import sys,pathlib,json,collections,hashlib
sys.path.insert(0,str(pathlib.Path(__file__).resolve().parent))
from rs3cache import *
from hdcache import osrs
from PIL import Image
c=Cache()
m=json.loads((ROOT/'materials.json').read_bytes())
flags=collections.Counter((r['layout'],r['flags']) for r in m.values())
result={'material_flags':[[v,hex(f),n] for (v,f),n in sorted(flags.items())]}
with (ROOT.parent/'research-npc/census/npcs.jsonl').open() as f:n=next(json.loads(l) for l in f if json.loads(l)['id']==12214)
proof=[]
for r in (239,242):
    h=HDCache(r)
    counts=collections.Counter()
    for i in n['models']:
        model=osrs.decode_model(h.group(7,i))
        counts.update(sum(w>0 for _,w in ws) for ws in (model['bone_weights'] or []))
    proof.append([r,n['models'],dict(counts)])
result['control_bones']=proof
im=Image.open(ROOT/'viewer-check.png').convert('RGB')
result['fixture_nonbackground']=[sum(v!=(32,39,44) for v in im.crop((x,y,x+220,y+218)).get_flattened_data()) for y in (25,265) for x in (0,220,440)]
assert all(result['fixture_nonbackground'])
(ROOT/'control-check.json').write_text(json.dumps(result,indent=1))
print(proof, result['fixture_nonbackground'])

