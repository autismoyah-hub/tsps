"""Rank possible effects in the same authoring batch. These are candidates, not encounter triggers."""
import sys,pathlib,json
sys.path.insert(0,str(pathlib.Path(__file__).parent))
from definitions import *
from models import decode
from math import log1p
c=Cache(614);o=Cache(239)
data=json.loads((ROOT/'study.json').read_text());spots=[]
for i,raw in records(c,'spot'):
 d=parse('spot',raw)
 if d['models'] and 'sequence' in d:
    spots.append((i,d['models'][0],d['sequence']))
control=[json.loads(l) for l in (ROOT.parent/'research-effects/data/spots.jsonl').open()]
out=[]
for boss in [r for r in data if r['kind']=='npc']:
 ids=[v[0] for v in boss['animations'].values()]
 def distance(row):
    return min(abs(row[1]-m) for m in boss['models']),min(abs(row[2]-a) for a in ids)
 best=sorted(spots,key=lambda r:sum(log1p(x) for x in distance(r)))
 print(boss['name'],[(r,distance(r)) for r in best[:2]])
 i,mid,seq=best[0];m=decode(c.group(7,mid))
 frames=definition(c,'sequence',seq).get('frames',0)
 candidates=[r for r in control if r.get('m') and r.get('s') and r['m']['faces'] and r['s'].get('frames')]
 def cost(r):
    return abs(log1p(r['m']['faces'])-log1p(len(m['faces'])))+abs(log1p(r['s']['frames'])-log1p(frames))+abs(r['m']['alpha_share']-sum(v!=0 for v in m['alpha'])/max(len(m['faces']),1))
 r=min(candidates,key=cost)
 hd=dict(vertices=len(m['vertices']),faces=len(m['faces']),parts=1,textures=sorted(set(m['face_textures'])-{-1}),textured=sum(t>=0 for t in m['face_textures']),alpha=sorted(set(m['alpha'])-{0}),colours=len(set(m['colors'])))
 ctrl=osrs.decode_model(o.group(7,r['model']))
 ref=dict(vertices=len(ctrl['vertices']),faces=len(ctrl['faces']),parts=1,textures=sorted(set(ctrl['face_textures'])-{-1}),textured=sum(t>=0 for t in ctrl['face_textures']),alpha=sorted(set(ctrl['alpha'])-{0}),colours=len(set(ctrl['colors'])))
 out.append(dict(kind='spot',id=i,name=f"Spot {i}",models=[mid],hd=hd,ref=dict(id=r['id'],name=r['name'],metrics=ref,anim={'sequence':[r['sequence'],r['s']['frames']]}),animations={'sequence':[seq,frames]},candidate_for=boss['name'],distance=distance(best[0])))
(ROOT/'spot-study.json').write_text(json.dumps(out,indent=1))

