"""Read-only inputs, per-definition assembly, shared classic renderer."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import base64
import json
from pathlib import Path
import subprocess
import sys
import tempfile
from functools import lru_cache

ROOT = Path(__file__).resolve().parent
ERA = ROOT.with_name('tools-hd-era')
sys.dont_write_bytecode = True
sys.path[:0] = [str(ROOT), str(ERA)]
import numpy as np
from PIL import Image
import hdview
import hdconvert
import itemdefs
from hdcache import Cache, Cursor, osrs
from materials import preview
from textures import sprite
from bake import NODE

KINDS = ['model', 'object', 'item', 'item-ground', 'item-worn', 'item-female', 'spot']

@lru_cache(None)
def cache(rev):
    return Cache(rev)

def target(rev, kind, number):
    c = cache(rev)
    if kind.startswith('item'):
        d = itemdefs.definition(c, 'item', number)
        wear = (23,24,78) if kind=='item-worn' else (25,26,79)
        worn = kind in ('item-worn','item-female')
        ids = [d['wear'][k] for k in wear if k in d.get('wear',{})] if worn else d['models']
        if not ids:
            raise ValueError(f'No {kind} model for {number}')
        parts = [hdview.asset(c,'model',i,True)[1][0] for i in ids]
        for m in parts:
            if worn:
                off = d.get('wear_offset',{}).get(125 if kind=='item-worn' else 126,(0,0,0))
                m['vertices'] = [tuple(x+y for x,y in zip(p,off)) for p in m['vertices']]
            else:
                m['vertices'] = [tuple(int(x*y/128) for x,y in zip(p,d['scale'])) for p in m['vertices']]
            for field,key in [('colors','recolor'),('face_textures','retexture')]:
                changes = dict(d[key])
                m[field] = [changes.get(x,x) for x in m[field]]
        d = dict(d,models=ids,presentation=kind)
    else:
        d,parts = hdview.asset(c,kind,number,True)
    return d,parts

def combined(parts):
    m = osrs.merge(parts)
    if any('uv' in p for p in parts):
        m['uv'] = np.concatenate([p.get('uv',np.zeros((len(p['faces']),3,2))) for p in parts])
    return m

def framing(vertices):
    v = np.asarray(vertices)
    mid = (v.min(0)+v.max(0))/2
    return [*mid, min(16,100/max(1,np.linalg.norm(v-mid,axis=1).max()))]

def render(m, rev, camera, modes=('lit',), raw=None):
    with tempfile.TemporaryDirectory(dir=ROOT) as tmp:
        p = Path(tmp)
        (p/'model.bin').write_bytes(hdview.packed([m]) if raw is None else raw)
        if 'uv' in m:
            (p/'projected.uv').write_bytes(m['uv'].astype('<f4').tobytes())
        for t in set(m['face_textures'])-{-1}:
            if rev == 242:
                rec = cache(242).files(9,0)[t]
                (p/f't{t}.def').write_bytes(rec)
                (p/f't{t}.sprite').write_bytes(cache(242).group(8,int.from_bytes(rec[:2],'big')))
            else:
                (p/f't{t}.rgba').write_bytes(preview(cache(rev),t)[0].tobytes())
        out = {}
        for mode in modes:
            subprocess.run([_AP.exe('sheet-render.exe', ROOT / 'target/release'),p,mode,*map(str,camera)],check=True)
            arr = np.stack([np.frombuffer((p/f'{i}.rgb').read_bytes(),np.uint8).reshape(256,256,3) for i in range(6)])
            if mode.split('-')[-1] in ('ids','alpha','groups'):
                rgb = arr.astype(np.int32)
                arr = (rgb[...,0]<<16)+(rgb[...,1]<<8)+rgb[...,2]-1
                if mode.endswith('ids'): arr-=1
            out[mode] = arr
        return out

def area(m):
    v = np.asarray(m['vertices'],float)[m['faces']]
    return np.linalg.norm(np.cross(v[:,1]-v[:,0],v[:,2]-v[:,0]),axis=1)/2

def shares(values,weights):
    values=np.asarray(values)
    return {int(k): float(weights[values==k].sum()/weights.sum()) for k in np.unique(values)}

def signature(image):
    a=np.asarray(image.convert('RGBA'),dtype=float)
    a[:,:,:3]=a[:,:,:3]*a[:,:,3:]/255+40*(1-a[:,:,3:]/255)
    return np.concatenate([np.quantile(a,[.1,.25,.5,.75,.9],axis=(0,1)).ravel(),a.mean(axis=(0,1))*2])

@lru_cache(None)
def catalogue():
    c=cache(242)
    return {t:signature(Image.fromarray(sprite(c.group(8,int.from_bytes(rec[:2],'big'))))) for t,rec in c.files(9,0).items()}

def conversion(rev,d,parts):
    """Use hdconvert for each raw mesh, reapply the definition and replace HD-only UV bases."""
    out=[]
    mapping={}
    for j,(i,p) in enumerate(zip(d['models'],parts)):
        raw,_=hdconvert.convert(rev,i)
        m=osrs.decode_model(raw)
        assert len(m['vertices'])==len(p['vertices']) and len(m['faces'])==len(p['faces'])
        wear = d.get('presentation') in ('item-worn','item-female')
        offset=next((row[1:] for row in d.get('offsets',[]) if row[0]==j),(0,0,0))
        scale=[128]*3 if wear else d.get('scale',[128]*3)
        post=d.get('offset',(0,0,0))
        if wear:
            post=d.get('wear_offset',{}).get(125 if d['presentation']=='item-worn' else 126,(0,0,0))
        def transform(v):
            x,y,z=(int((a+b)*s/128) for a,b,s in zip(v,offset,scale))
            if d.get('mirror'):z=-z
            for _ in range(d.get('rotation',0)//90%4):x,z=z,-x
            return (x+post[0],y+post[1],z+post[2])
        m['vertices']=[transform(v) for v in m['vertices']]
        if d.get('mirror'):m['faces']=[(c,b,a) for a,b,c in m['faces']]
        for field,key in [('colors','recolor'),('face_textures','retexture')]:
            substitutions=dict(d.get(key,[]))
            m[field]=[substitutions.get(x,x) for x in m[field]]
        for field in ('vertices','faces','colors','face_textures'):
            if m[field]!=p[field]:raise ValueError(f'conversion disagrees after assembly: {i} {field}')
        for t in set(m['face_textures'])-{-1}:
            if t not in mapping:
                sig=signature(preview(cache(rev),t)[0])
                mapping[t]=min(catalogue(),key=lambda k: (float(np.abs(sig-catalogue()[k]).mean()),k))
        m['face_textures']=[mapping.get(t,t) for t in m['face_textures']]
        # Keep simple bases, assign the face itself to projected selectors.
        simple=[j for j,t in enumerate(m['texture_types']) if t==0]
        remap={old:new for new,old in enumerate(simple)}
        m.update(texture_coords=[remap.get(j,-1) for j in m['texture_coords']],
                 texture_types=[0]*len(simple),texture_triangles=[m['texture_triangles'][j] for j in simple],
                 complex_sections=['']*5)
        out.append(m)
    return out,mapping

def animation(rev,seq,model):
    c=cache(rev)
    r=Cursor(c.files(20,seq>>7)[seq&127])
    op14=False
    while (op:=r.u8())!=1:
        if op==14: op14=True
        if op==3:r.take(r.u8())
        else:r.take({4:0,14:0,15:0,16:0,5:1,8:1,9:1,10:1,11:1,2:2,6:2,7:2}[op])
    n=r.u16()
    holds=[r.u16() for _ in range(n)]
    lo=[r.u16() for _ in range(n)]
    frames=[c.files(0,r.u16())[f] for f in lo]
    # Later sequence fields can contain arbitrary operand bytes. Walk boundaries.
    while (op:=r.u8())!=0:
        if op==14:op14=True
        if op==3:r.take(r.u8())
        elif op==12:r.take(4*r.u8())
        elif op==13:
            for _ in range(r.u16()):
                count=r.u8()
                if count:r.take(3+2*(count-1))
        else:r.take({4:0,14:0,15:0,16:0,5:1,8:1,9:1,10:1,11:1,2:2,6:2,7:2,18:0,19:2,20:5}[op])
    offset=int(rev>=610)
    ids={int.from_bytes(f[offset:offset+2],'big') for f in frames}
    enc=lambda b:base64.b64encode(b).decode()
    data=dict(rev=rev,model=enc(hdview.packed([model])),op14=op14,frames=[enc(f) for f in frames],
              bases={i:enc(c.files(1,i)[0]) for i in ids})
    proc=subprocess.run([NODE,ROOT/'pose.cjs'],input=json.dumps(data).encode(),stdout=subprocess.PIPE,check=True)
    posed=[]
    for frame in json.loads(proc.stdout):
        m=dict(model,vertices=[tuple(row) for row in frame['vertices']])
        for key in ('alpha','colors'):
            if frame[key]:m[key]=frame[key]
        posed.append(m)
    return holds,posed

