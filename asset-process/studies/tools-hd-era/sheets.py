import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys,pathlib,json
sys.path.insert(0,str(pathlib.Path(__file__).parent))
from hdview import *
from PIL import ImageFont
c=Cache(614);o=Cache(239);folder=ROOT/'sheets';folder.mkdir(exist_ok=True)
study=json.loads((ROOT/'study.json').read_text())+json.loads((ROOT/'spot-study.json').read_text())
def catalog(path):return {d['id']:d for d in map(json.loads,path.open(encoding='utf8'))}
npc=catalog(ROOT.parent/'research-npc/census/npcs.jsonl');items=catalog(ROOT.parent/'research-items/data/items.jsonl');spots=catalog(ROOT.parent/'research-effects/data/spots.jsonl')
arch=Path(_AP.fix('<REFERENCE_ARCHIVE>/objects'))
def reference(r):
 k=r['kind'];i=r['ref']['id'];swaps=[];textures=[];scale=[128]*3
 if k=='npc':
    d=npc[i];ids=d['models'];swaps=d['recolor'];scale=[d['scale'][0],d['scale'][1],d['scale'][0]]
 elif k=='item':
    d=items[i];ids=[d['model']];swaps=d['recolor'];textures=d['retexture'];scale=d['scale']
 elif k=='object':
    shape=definition(c,k,r['id'])['shape']
    d=json.loads((arch/f'shape-{shape}'/f'{i}.json').read_text());ids=d['model_ids'];swaps=d['recolors'];textures=d['retextures'];scale=d['scale']
 else:ids=[spots[i]['model']]
 parts=[osrs.decode_model(o.group(7,j)) for j in ids]
 for m in parts:
    m['vertices']=[tuple(int(v[k]*scale[k]/128) for k in range(3)) for v in m['vertices']]
    for field,op in [('colors',swaps),('face_textures',textures)]:
        lookup=dict(op);m[field]=[lookup.get(x,x) for x in m[field]]
 return views(o,k,i,({},parts))[1][-1]
def font(size,bold=False):
 return ImageFont.truetype(_AP.fix('<FONT_DIR>/')+('segoeuib.ttf' if bold else 'segoeui.ttf'),size)
def row(img,x,y,r):
 d,ims,issues=views(c,r['kind'],r['id']);im=ims[-1]
 ref=reference(r)
 pen=ImageDraw.Draw(img)
 for j,(pic,name,stat,id,rev) in enumerate([(im,r['name'],r['hd'],r['id'],614),(ref,r['ref']['name'],r['ref']['metrics'],r['ref']['id'],239)]):
    x0=x+j*365
    pic.thumbnail((150,150));img.paste(pic,(x0+7,y+25))
    pen.text((x0+8,y+2),f'r{rev}  {id}   {name[:27]}',font=font(14,True),fill='#f2f3f6')
    lines=[f"{stat['faces']:,} faces",f"{stat['vertices']:,} vertices",f"{len(stat['textures'])} textures  /  {stat['textured']} faces",f"{stat['parts']} parts  /  {stat['colours']} colours",
           'Alpha  '+(','.join(map(str,stat['alpha'][:5])) or 'opaque')+('…' if len(stat['alpha'])>5 else '')]
    pen.multiline_text((x0+163,y+39),'\n'.join(lines),font=font(13),fill='#ced7de',spacing=4)
 pen.line((x,y+183,x+725,y+183),fill='#434b52')
 return issues
titles={'npc':'Bosses and large creatures','object':'Textured, translucent scenery','item':'Textured inventory models','spot':'Animated spot effects'}
md=['# Paired study sheets','','All HD examples are from revision 614. Vertices count stored positions across parts before welding. Colours count distinct post-recolour HSL words, including textured faces. Alpha lists distinct nonzero stored bytes. Texture IDs belong to each revision. Each pair is framed independently.','']
for kind,title in titles.items():
 chosen=[r for r in study if r['kind']==kind]
 for pg in range(0,len(chosen),8):
    page=chosen[pg:pg+8];board=Image.new('RGB',(750,72+len(page)*184),'#262a30');pen=ImageDraw.Draw(board)
    pen.text((15,7),title,font=font(23,True),fill='white')
    pen.text((15,41),'RS2 / HD 614                                        OSRS 239     •     classic client, bind pose',font=font(14),fill='#9dbdc5')
    file=f'{kind}-{pg//8+1}.png';md.extend([f'## {title} · {pg//8+1}','','[Open paired sheet](sheets/'+file+')','',
       '| Revision / ID | Name | Parts | Vertices | Faces | Textures (IDs) | Textured faces | Nonzero alpha bytes | Colours | Sequences (ID / frames) |',
       '|---|---|---:|---:|---:|---|---:|---|---:|---|'])
    notes=[]
    for j,r in enumerate(page):
        issues=row(board,10,69+j*184,r)
        for rev,d,m in [(614,r,r['hd']),(239,r['ref'],r['ref']['metrics'])]:
            anim=d.get('animations',d.get('anim',{}))
            md.append('| '+' | '.join(str(v) for v in [f"{rev} / {d['id']}",d['name'],m['parts'],m['vertices'],m['faces'],','.join(map(str,m['textures'])) or '—',m['textured'],','.join(map(str,m['alpha'])) or '—',m['colours'],', '.join(f'{s} {a}/{n}' for s,(a,n) in anim.items()) or '—'])+' |')
        if kind=='npc':notes.append(f"{r['name']} (NPC {r['id']}) has combat level {r['combat']}, footprint {r['size']} and model IDs {r['models']}.")
        if kind=='spot':notes.append(f"Spot {r['id']} is a batch-neighbour candidate for {r['candidate_for']}. Absolute ID gaps (model, sequence) are {r['distance']}. Encounter use is unverified.")
        if any(':' in s for s in issues):notes.append(f"{kind} {r['id']} uses diagnostic tiles for "+', '.join(s for s in issues if ':' in s)+'.')
    board.save(folder/file);md+=['']+notes+[''];print(file,flush=True)
(ROOT/'STUDY.md').write_text('\n'.join(md),encoding='utf8')

