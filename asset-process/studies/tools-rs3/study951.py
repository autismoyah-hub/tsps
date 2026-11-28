"""Build paired definition tables and orthographic OSRS control sheets."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys,pathlib,json,sqlite3,collections,math
sys.path.insert(0,str(pathlib.Path(__file__).resolve().parent))
from rs3cache import Cache,HDCache,ROOT
from hdcache import osrs
import numpy as np
from PIL import Image,ImageDraw,ImageFont
db=sqlite3.connect(ROOT/'assets.sqlite')
cache=Cache()
osdisk=HDCache(239)
files={'npc':'research-npc/census/npcs.jsonl','item':'research-items/data/items.jsonl','spot':'research-effects/data/spots.jsonl'}
def read_catalog(k):
    with (ROOT.parent/files[k]).open(encoding='utf8') as f:
        return [json.loads(line) for line in f]
catalog={k:read_catalog(k) for k in files}
groups={
'Creatures':('npc',[(27352,'The Leviathan'),(25593,'Zulrah'),(26062,'Abyssal Sire'),(28069,'Xarpus'),(28526,'TzKal-Zuk'),(25513,'Bryophyta'),(30690,'Vorkath'),(25662,'King Black Dragon'),(28392,'Hespori'),(28239,'Ice giant')]),
'Weapons':('item',[('Eldritch crossbow','Zaryte crossbow'),('Fractured Staff of Armadyl','Staff of the dead'),('Dark Shard of Leng','Blade of saeldor'),('Noxious scythe','Scythe of vitur'),('Noxious staff','Toxic staff of the dead'),('Noxious longbow','Twisted bow'),('Dragon rider lance','Dragon hunter lance'),('Staff of Sliske','Nightmare staff'),('Bow of the Last Guardian','Bow of faerdhinen'),('Roar of Awakening','Tumeken\'s shadow')]),
'Armour':('item',[('Tectonic mask','Ancestral hat'),('Tectonic robe top','Ancestral robe top'),('Tectonic robe bottom','Ancestral robe bottom'),('Sirenic mask','Masori mask'),('Sirenic hauberk','Masori body'),('Sirenic chaps','Masori chaps'),('Vestments of havoc hood','Inquisitor\'s great helm'),('Vestments of havoc robe top','Inquisitor\'s hauberk'),('Vestments of havoc robe bottom','Inquisitor\'s plateskirt'),('Vestments of havoc boots','Primordial boots')]),
'Effects':('spot',[(90,90),(91,91),(92,92)])}
def parts_for(k,r):
    ids=r['models'] if k=='npc' else [r['model']]
    parts=[osrs.decode_model(osdisk.group(7,i)) for i in ids]
    for p in parts:
        for field,key in [('colors','recolor'),('face_textures','retexture')]:
            swaps=r.get(key)
            if isinstance(swaps,list):
                remap=dict(swaps)
                p[field]=[remap.get(v,v) for v in p[field]]
    return parts
def preview(parts):
    polys=[]
    for p in parts:
        pts=np.asarray(p['vertices'],float)
        x,y,z=pts.T
        screen=np.column_stack((.85*x+.53*z,y*.94+.34*(-.53*x+.85*z),-.53*x+.85*z-.34*y))
        for f,col,a in zip(p['faces'],p['colors'],p['alpha']):
            if a>=254:continue
            tri=screen[f,:] if isinstance(f,list) else screen[list(f),:]
            normal=np.cross(tri[1]-tri[0],tri[2]-tri[0])
            l=np.linalg.norm(normal)
            shade=.5+.5*abs(normal@np.array([.3,-.7,.64]))/max(1,l)
            rgb=osrs.hsl_to_rgb(col)
            rgb=tuple(int(max(0,min(255,v*shade*255))) for v in rgb[:3])
            polys.append((tri[:,2].mean(),tri[:,:2],(*rgb,255-a)))
    image=Image.new('RGB',(160,124),'#20272c')
    if not polys:return image
    coords=np.concatenate([t for _,t,_ in polys])
    lo,hi=coords.min(0),coords.max(0)
    scale=min(147/max(1,hi[0]-lo[0]),113/max(1,hi[1]-lo[1]))
    pen=ImageDraw.Draw(image,'RGBA')
    for _,tri,col in sorted(polys,key=lambda v:v[0]):
        q=(tri-(lo+hi)/2)*scale+[80,62]
        pen.polygon(list(map(tuple,q)),fill=col)
    return image
def control(k,key):
    can=[r for r in catalog[k] if (r['id']==key if isinstance(key,int) else r['name'].casefold()==key.casefold()) and (r.get('models') if k=='npc' else r.get('model',-1)>=0)]
    if k=='npc':can.sort(key=lambda r:('Attack' not in (r.get('ops') or []),-r.get('combat',0)))
    return can[0]
font=ImageFont.truetype(_AP.fix('<FONT_DIR>/segoeui.ttf'),16)
rows=[]
md=['# Build 951 and OSRS 239 study','',
'RS3 columns contain direct model IDs from the definition. Faces, vertices, materials, textures, transparency and bones per vertex are unavailable for every RS3 row because archive 47 contains zero resident groups. OSRS columns measure the decoded inventory or body meshes before welding. Texture count is distinct assigned texture IDs after available item swaps. Alpha counts faces with a nonzero transparency byte. Bones is the maximum number of explicit nonzero influences on any vertex, excluding classic labels.','',
'Controls match encounter role, body plan, weapon type or armour slot. The sheets show the OSRS controls only, with an independently framed static triangle preview. Shading uses face colour and alpha without texture sampling, animation, definition scaling or game lighting. The creature selections identify modern encounters by name. Release dates are not recorded in the definitions.','']
for title,(kind,pairs) in groups.items():
    board=Image.new('RGB',(880,55+len(pairs)*128),'#20272c')
    pen=ImageDraw.Draw(board)
    pen.text((15,12),title+'  |  Build 951 definition and OSRS 239 control',font=font,fill='white')
    md.extend(['## '+title,'',f'[OSRS comparison sheet]({title.lower()}.png)','','| 951 definition | RS3 model IDs | 239 control | Faces | Vertices | Textures | Alpha faces | Max bones |','|---|---|---|---:|---:|---:|---:|---:|'])
    for i,(key,ref) in enumerate(pairs):
        col='id' if isinstance(key,int) else 'name'
        hit=db.execute(f"select id,name,models,properties from assets where kind=? and {col}=? collate nocase and models!='[]' order by id limit 1",(kind,key)).fetchone()
        if not hit:raise ValueError(('missing RS3',key))
        rid,name,models,properties=hit
        models=json.loads(models)
        assert all(x in cache.index(47) for x in models if x!=-1)
        r=control(kind,ref)
        parts=parts_for(kind,r)
        stats=[sum(len(p['faces']) for p in parts),sum(len(p['vertices']) for p in parts),len({t for p in parts for t in p['face_textures'] if t>=0}),sum(bool(a) for p in parts for a in p['alpha']),max((sum(w>0 for _,w in v) for p in parts for v in (p['bone_weights'] or [])),default=0)]
        name=name or 'Spot '+str(rid)
        label=f"{r['name']} ({r['id']})"
        mids=', '.join(map(str,models))
        md.append('| '+' | '.join(map(str,[f'{name} ({rid})',mids,label,*stats]))+' |')
        y=55+i*128
        pen.line((12,y,868,y),fill='#404b53')
        pen.text((15,y+20),f'951   {name} ({rid})',font=font,fill='#f5e8c0')
        pen.text((15,y+55),'Models  '+mids,font=font,fill='#c7d0d6')
        board.paste(preview(parts),(390,y+2))
        pen.text((560,y+20),label,font=font,fill='white')
        pen.text((560,y+55),f'{stats[0]:,} faces   {stats[1]:,} vertices',font=font,fill='#c7d0d6')
        rows.append(dict(category=title,id=rid,name=name,models=models,properties=json.loads(properties),control=r['id'],control_name=r['name'],osrs=stats))
    md.append('')
    board.save(ROOT/(title.lower()+'.png'))
md+=['The three effect rows use same-ID controls. The TSPS CombatSpells.WIND_STRIKE implementation uses 90 at cast, 91 for the projectile and 92 at impact. The 951 definitions link 91 and 92 to a single model with different sequences. Encounter use in 951 remains unverified.']
(ROOT/'STUDY.md').write_text('\n'.join(md),encoding='utf8')
(ROOT/'pairs.json').write_text(json.dumps(rows,indent=1))

