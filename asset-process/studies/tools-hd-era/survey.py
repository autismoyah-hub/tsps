import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys,pathlib,json,collections
sys.path.insert(0,str(pathlib.Path(__file__).parent))
from hdview import *
from definitions import records,parse
from functools import lru_cache
c=Cache(614);o=Cache(239)
def rows(p):
 return (json.loads(line) for line in p.open(encoding='utf-8'))
builds=ROOT.parent
items={};item_ids={}
for d in rows(builds/'research-items/data/items.jsonl'):
 item_ids[d['id']]=d
 if d.get('model') and d['note'][0]<0 and d['placeholder'][0]<0:items.setdefault(d['name'].lower(),d)
npcs={}
for d in rows(builds/'research-npc/census/npcs.jsonl'):
 if d['models'] and d['combat']>npcs.get(d['name'],{}).get('combat',-1):npcs[d['name']]=d
seqs={d['id']:d for d in rows(builds/'research-npc/census/sequences.jsonl')}
def metrics(parts):
 return dict(vertices=sum(len(m['vertices']) for m in parts),faces=sum(len(m['faces']) for m in parts),
             parts=len(parts),textures=sorted({v for m in parts for v in m['face_textures'] if v>=0}),
             textured=sum(v>=0 for m in parts for v in m['face_textures']),
             alpha=sorted({v for m in parts for v in m['alpha'] if v}),
             colours=len({v for m in parts for v in m['colors']}))
def refmetrics(ids,recolor=(),retexture=()):
 parts=[osrs.decode_model(o.group(7,i)) for i in ids]
 for m in parts:
    for k,rs in [('colors',recolor),('face_textures',retexture)]:
        swap=dict(rs);m[k]=[swap.get(v,v) for v in m[k]]
 return metrics(parts)
def bas(d):
 r=Cursor(c.files(2,32)[d['bas']]);a={}
 while op:=r.u8():
    if op==1:a.update(stand=r.u16(),walk=r.u16())
    elif op==52:
        for _ in range(r.u8()):a[f'idle{_+1}']=r.u16();r.u8()
    elif op in (26,):r.take(2)
    elif op in (*range(2,10),*range(38,52)):r.take(2)
    else:raise ValueError(('BAS',op))
 return {k:[i,definition(c,'sequence',i).get('frames',0)] for k,i in a.items() if i!=65535}
def record(kind,i,ref,hd,measure,anims=None):
 return dict(kind=kind,id=i,name=hd['name'],models=hd['models'],combat=hd.get('combat'),size=hd.get('size'),
             hd=measure,ref=ref,animations=anims or {})
data=[]
for i,name in [(11886,"K'ril Tsutsaroth"),(8528,'Nomad'),(9964,'Corporeal Beast'),(9911,"K'ril Tsutsaroth"),(10156,'Sarachnis'),(10141,'General Graardor'),(10057,'Ice giant'),(9176,'Skeletal wyvern'),(11939,'Abyssal Sire'),(8596,'Avatar of Destruction')]:
 d,parts=asset(c,'npc',i);r=next(v for n,v in npcs.items() if n.lower()==name.lower())
 ref=dict(id=r['id'],name=r['name'],metrics=refmetrics(r['models'],r['recolor']),anim={s:[a,seqs.get(a,{}).get('frames')] for s,a in zip(['stand','walk'],[r['anims'][0],r['anims'][3]]) if a>=0})
 data.append(record('npc',i,ref,d,metrics(parts),bas(d)))
# Prefer distinct same-name contemporary items with readable textures; draw the rest with marked preview limitations.
unsupported=set(map(int,json.loads((ROOT/'texture-errors-614.json').read_text())))
candidates=[];names=set();models=set()
for i,raw in records(c,'item'):
 d=parse('item',raw)
 same=item_ids.get(i,{})
 r=same if same.get('model') and same.get('name','').lower()==d['name'].lower() else items.get(d['name'].lower())
 if not r or not d['models'] or d['name'] in names or d['models'][0] in models:continue
 _,parts=asset(c,'item',i);m=metrics(parts)
 if not m['textures']:continue
 names.add(d['name']);models.add(d['models'][0])
 if r.get('wear_positions',[-1])[0]<0 and not any(term in d['name'].lower() for term in ['shield','sword','staff','bow','cape','helm','armour','ring','amulet','robe','plate','boots','gloves','mace']):continue
 candidates.append((bool(set(m['textures'])&unsupported),-len(m['textures']),-m['textured'],i,d,m,r))
for _,__,___,i,d,m,r in sorted(candidates)[:20]:
 ref=dict(id=r['id'],name=r['name'],metrics=refmetrics([r['model']],r['recolor'],r['retexture']))
 data.append(record('item',i,ref,d,m))
# Standalone scenery, direct same-ID same-name controls where present.
arch=Path(_AP.fix('<REFERENCE_ARCHIVE>/objects'))
seen=set();count=0;pool=[]
@lru_cache(maxsize=512)
def object_metrics(ids,recolor,retexture):
 parts=[decode(c.group(7,i)) for i in ids]
 for m in parts:
    for k,rs in [('colors',recolor),('face_textures',retexture)]:
        mapping=dict(rs);m[k]=[mapping.get(v,v) for v in m[k]]
 return metrics(parts)
for i,raw in records(c,'object'):
 d=parse('object',raw)
 if not d['name'] or not d['models'] or d['name'] in seen:continue
 m=object_metrics(tuple(d['models']),tuple(d['recolor']),tuple(d['retexture']))
 if not m['textures'] or not any(0<a<254 for a in m['alpha']):continue
 pool.append((len(m['textures']),m['textured'],i,d,m))
for _,__,i,d,m in sorted(pool,key=lambda v:(v[0],v[1]),reverse=True):
 if d['name'].lower() in ('null','') or d['name'] in seen:continue
 p=arch/f"shape-{d['shape']}"/f'{i}.json'
 if not p.exists():continue
 r=json.loads(p.read_text())
 if r['name'].lower()!=d['name'].lower() or not r['model_ids']:continue
 ref=dict(id=i,name=r['name'],metrics=refmetrics(r['model_ids'],r['recolors'],r['retextures']))
 data.append(record('object',i,ref,d,m));seen.add(d['name']);count+=1
 if count==6:break
(ROOT/'study.json').write_text(json.dumps(data,indent=1))
print(collections.Counter(r['kind'] for r in data))

