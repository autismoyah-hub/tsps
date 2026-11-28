"""Draw six classic views. Usage: hdview.py ID OUT.png --cache 614 --kind npc"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys,pathlib
sys.dont_write_bytecode=True
sys.path.insert(0,str(pathlib.Path(__file__).resolve().parent))
from models import *
from definitions import definition
from materials import preview
from PIL import Image,ImageDraw
import argparse, json, subprocess, tempfile
import model_packing

def asset(c,kind,i,with_uv=False):
 d=dict(models=[i],name=f'Model {i}') if kind=='model' else definition(c,kind,i)
 if not d['models']:raise ValueError(f'{kind} {i} has no direct models')
 parts=[];scale=d.get('scale',[128]*3)
 for j,mid in enumerate(d['models']):
    raw=c.group(7,mid)
    m=osrs.decode_model(raw) if c.rev==239 else decode(raw)
    offset=next((row[1:] for row in d.get('offsets',[]) if row[0]==j),(0,0,0))
    # Later versioned models use four subunits per legacy unit.
    div=4 if m.get('hd',{}).get('version') in (14,15) else 1
    if with_uv and any(m['texture_types']) and c.rev!=239:
        from mapping import uvmap
        m['uv']=uvmap(m)
    m['vertices']=[tuple(int((v[k]//div+offset[k])*scale[k]/128) for k in range(3)) for v in m['vertices']]
    if d.get('mirror'):
        m['vertices']=[(x,y,-z) for x,y,z in m['vertices']]
        m['faces']=[(z,y,x) for x,y,z in m['faces']]
        if 'uv' in m:m['uv']=m['uv'][:,::-1,:]
    for _ in range(d.get('rotation',0)//90%4):
        m['vertices']=[(z,y,-x) for x,y,z in m['vertices']]
    if 'offset' in d:
        m['vertices']=[tuple(v[k]+d['offset'][k] for k in range(3)) for v in m['vertices']]
    for field,key in [('colors','recolor'),('face_textures','retexture')]:
        swap=dict(d.get(key,[]));m[field]=[swap.get(v,v) for v in m[field]]
    parts.append(m)
 return d,parts
def packed(parts):
 g=osrs.merge(parts)
 if g['vertex_skins'] is None:del g['vertex_skins']
 if not g['has_face_skin']:del g['face_skins']
 plan=model_packing.canonical_plan(g,g['bone_weights'])
 return model_packing.encode(g,plan,g['complex_sections'])
def views(c,kind,i,assembled=None):
 d,parts=assembled if assembled else asset(c,kind,i,with_uv=True)
 with tempfile.TemporaryDirectory(dir=ROOT) as tmp:
    p=Path(tmp)
    # For a raw OSRS reference, pass the original file directly to the Rust decoder.
    raw=c.group(7,i) if c.rev==239 and kind=='model' else packed(parts)
    (p/'model.bin').write_bytes(raw)
    issues=[]
    for tid in sorted({t for m in parts for t in m['face_textures'] if t>=0}):
        if c.rev==239:
            from textures import sprite
            rec=c.files(9,0)[tid];im=Image.fromarray(sprite(c.group(8,int.from_bytes(rec[:2],'big'))));err=None
        else:im,err=preview(c,tid)
        if err:issues.append(f't{tid}: {err}')
        (p/f't{tid}.rgba').write_bytes(im.tobytes())
    if any('uv' in m for m in parts):
        import numpy as np
        uv=np.concatenate([m['uv'] if 'uv' in m else np.zeros((len(m['faces']),3,2)) for m in parts])
        (p/'projected.uv').write_bytes(uv.astype('<f4').tobytes())
        if any(m.get('degenerate_uv_faces') for m in parts):issues.append('degenerate UV fallback')
    subprocess.run([str(_AP.exe('hdclassic.exe', ROOT / 'target/release')),str(p)],check=True)
    imgs=[Image.frombytes('RGB',(256,256),(p/f'{k}.rgb').read_bytes()) for k in range(6)]
 return d,imgs,issues
def sheet(c,kind,i,path):
 d,imgs,issues=views(c,kind,i)
 im=Image.new('RGB',(768,585),'#262a30');draw=ImageDraw.Draw(im)
 draw.text((12,7),f"r{c.rev}   {kind} {i}   {d['name']}",fill='white')
 for k,(label,img) in enumerate(zip(['Front','Right','Back','Left','Top','Three-quarter'],imgs)):
    x=(k%3)*256;y=(k//3)*274+24
    im.paste(img,(x,y));draw.text((x+10,y+251),label,fill='#cdd4dc')
 if issues:draw.text((10,565),'Preview limitations: '+', '.join(issues)[:110],fill='#ffcf96')
 im.save(path)
if __name__=='__main__':
 p=argparse.ArgumentParser(description=__doc__);p.add_argument('id',type=int);p.add_argument('out',type=Path)
 p.add_argument('--cache',type=int,choices=[561,601,614],default=614)
 p.add_argument('--kind',choices=['model','object','npc','item'],default='model')
 a=p.parse_args();sheet(Cache(a.cache),a.kind,a.id,a.out)

