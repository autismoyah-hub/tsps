"""Survey the model(s) selected by the reference archive for each definition and shape."""
import sys, json, functools, struct
from pathlib import Path
from collections import defaultdict, Counter
from PIL import Image, ImageDraw
import numpy as np
HERE = Path(__file__).resolve().parent
ROOT = HERE.parents[1]
sys.path[:0] = [str(ROOT/'tools'), str(ROOT/'studies/research-animation')]
import osrscache as o, scene, animlib

def board():
    cache = animlib.Cache(o.CACHE)
    defs = cache.files(9,0)
    im = Image.new('RGB',(8*108, ((len(defs)+7)//8)*124),'#202328')
    draw = ImageDraw.Draw(im)
    for k,(tid,b) in enumerate(sorted(defs.items())):
        sprite,avg,opaque,direction,speed = struct.unpack('>HHBBB', b)
        raw = cache.file(8,sprite,0)
        count = int.from_bytes(raw[-2:],'big')
        meta = len(raw)-7-count*8
        w,h,n = struct.unpack_from('>HHB',raw,meta)
        x,y,sw,sh = (struct.unpack_from('>H',raw,meta+5+i*2*count)[0] for i in range(4))
        palette = np.vstack(([0,0,0],np.frombuffer(raw[meta-3*n:meta],np.uint8).reshape(n,3))).astype(np.uint8)
        idx = np.frombuffer(raw[1:1+sw*sh],np.uint8)
        idx = idx.reshape(sw,sh).T if raw[0]&1 else idx.reshape(sh,sw)
        canvas = Image.new('RGB',(w,h))
        canvas.paste(Image.fromarray(palette[idx]),(x,y))
        px,py = k%8*108+6,k//8*124+4
        im.paste(canvas.resize((96,96),Image.Resampling.NEAREST),(px,py))
        draw.text((px,py+98),str(tid),fill='white')
    im.save(HERE/'board-textures.png')
    print('Texture IDs',sorted(defs),flush=True)

@functools.cache
def material(mid):
    m = o.model(mid)
    return set(m['face_textures'])-{-1}, set(m['alpha'])-{0}

def census():
    bands = {}
    candidates = []
    for folder in sorted([*scene.ARCH.glob('objects/shape-*'), *scene.ARCH.glob('items/slot-*')]):
        kind = folder.name
        count = Counter()
        textures, alphas = set(),set()
        for p in folder.glob('*.json'):
            j = json.loads(p.read_bytes())
            if kind.startswith('slot'):
                ids = [i for i in (j.get('wear_models') or [[]])[0] if i not in (-1,None)]
            else:
                ids = j['model_ids']
                if j.get('model_shapes'):
                    ids = [i for i,s in zip(ids,j['model_shapes']) if s==int(kind[6:])]
            if not ids: continue
            t,a=set(),set()
            for mid in ids:
                ts,vs = material(mid)
                t.update(ts); a.update(vs)
            for old,new in j.get('retextures') or []:
                if old in t: t.remove(old); t.add(new)
            count['total']+=1; count['textured']+=bool(t); count['alpha']+=bool(a)
            count['both']+=bool(t and a); textures.update(t); alphas.update(a)
            if len(ids)==1:
                candidates.append([kind,j['id'],j.get('name'),ids[0],sorted(t),sorted(a)])
        bands[kind]=dict(count,ids=sorted(textures),values=sorted(alphas))
        (HERE/'bands.json').write_text(json.dumps(bands,indent=1))
        print(kind, count,flush=True)
        (HERE/'candidates.json').write_text(json.dumps(candidates))
if __name__=='__main__':
    board()
    census()
