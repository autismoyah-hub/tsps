"""Assemble decoded build-951 models by definition and export them for Blender / the gate.

kinds: model:ID, item:ID (inventory), item-worn:ID (23,24 male), item-female:ID (25,26), npc:ID.
Definition recolours (op 40) apply to vertex HSL, retextures (op 41) to group materials.
Material -> diffuse image: first texture reference of the material record (materials.json), PNG from archive 53.
"""
import sys, json, sqlite3, shutil, colorsys, pathlib
from functools import lru_cache
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
sys.dont_write_bytecode = True
import numpy as np
from PIL import Image
from rs3flat import Cache, ROOT
import rs3model

TEX = ROOT/'textures'/'diffuse'

@lru_cache(None)
def cache():
    return Cache()

@lru_cache(None)
def materials():
    return json.loads((ROOT/'materials.json').read_text())

def diffuse(material):
    row = materials().get(str(material))
    if not row or not row.get('references'): return None
    t = row['references'][0][2]
    p = TEX/f'{t}.png'
    if not p.exists():
        import textures951 as tx
        try:
            body, _, _ = tx.payload(cache(), 53, t)
            TEX.mkdir(parents=True, exist_ok=True); p.write_bytes(body)
        except Exception:
            return None
    return p

def model(i):
    return rs3model.decode(cache().group(47, i))

def definition(kind, i):
    db = sqlite3.connect((ROOT/'assets.sqlite').as_uri()+'?mode=ro', uri=True)
    base = 'item' if kind.startswith('item') else kind
    row = db.execute('select name, models, properties from assets where kind=? and id=?', (base, i)).fetchone()
    if not row: raise KeyError((kind, i))
    name, models, props = row[0], json.loads(row[1]), json.loads(row[2])
    if kind == 'item-worn': models = [props[k] for k in ('23', '24') if k in props]
    if kind == 'item-female': models = [props[k] for k in ('25', '26') if k in props]
    return dict(name=name or f'{kind} {i}', models=[m for m in models if m >= 0],
                recolour=props.get('recolour', []), retexture=props.get('retexture', []))

def hsl_rgb(c):
    h, s, l = (c >> 10) & 63, (c >> 7) & 7, c & 127
    return colorsys.hls_to_rgb(h/64 + 1/128, l/128, s/8 + 1/16)

def parts(spec):
    kind, _, num = spec.partition(':')
    if not num: kind, num = 'model', kind
    i = int(num)
    d = dict(name=f'Model {i}', models=[i], recolour=[], retexture=[]) if kind == 'model' else definition(kind, i)
    out = []
    rc, rt = dict(d['recolour']), dict(d['retexture'])
    for mid in d['models']:
        m = model(mid)
        m['model_id'] = mid
        m['colours'] = np.array([rc.get(int(c), int(c)) for c in m['colours']])
        for g in m['groups']:
            g['material'] = rt.get(g['material'], g['material'])
        out.append(m)
    return d, out

def osrs_faces(m):
    """Per-face arrays in OSRS axes and units (floats), winding reversed for the y mirror."""
    p = np.asarray(m['positions'], float)
    conv = np.stack([p[:, 0]/4, -p[:, 1]/4, p[:, 2]/4], 1)
    tris, mats, trans = [], [], []
    alpha = m.get('alpha', np.full(len(p), 255))
    for g in m['groups']:
        t = g['triangles'][:, ::-1]
        tris.append(t); mats += [g['material']]*len(t)
        trans.append(255 - np.round(alpha[t].mean(1)).astype(int))
    tris = np.concatenate(tris) if tris else np.zeros((0, 3), int)
    return conv, tris, np.array(mats, int), (np.concatenate(trans) if trans else np.zeros(0, int))

def export_json(spec, folder):
    """Write interchange JSON for rs3blend.py plus one PNG per textured material."""
    folder = pathlib.Path(folder); folder.mkdir(parents=True, exist_ok=True)
    d, ps = parts(spec)
    V, F, RGBA, UV, MAT, ALPHA, W = [], [], [], [], [], [], []
    base = 0
    for m in ps:
        p = np.asarray(m['positions'], float)
        V += p.tolist()
        RGBA += [list(hsl_rgb(int(c))) + [1.0] for c in m['colours']]
        a = m.get('alpha', np.full(len(p), 255))
        for g in m['groups']:
            for t in g['triangles']:
                F.append([int(x)+base for x in t])
                UV.append([[float(m['uv'][x][0]), 1-float(m['uv'][x][1])] for x in t])
                MAT.append(int(g['material']))
                ALPHA.append(int(255 - round(float(a[t].mean()))))
        if 'bones' in m:
            W += [[[int(b), min(255, int(w))] for b, w in zip(bs, ws)] for bs, ws in zip(m['bones'], m['weights'])]
        else:
            W += [[] for _ in range(len(p))]
        base += len(p)
    used = sorted({x for x in MAT if x >= 0})
    missing = []
    for mid in used:
        src = diffuse(mid)
        # Diffuse alpha is not opacity in many materials (vertex alpha carries translucency): write RGB.
        if src: Image.open(src).convert('RGB').save(folder/f'{mid}.png')
        else: missing.append(mid)
    # Materials without a diffuse image render with vertex colour only.
    MAT = [(-1 if x in missing else x) for x in MAT]
    j = dict(name=d['name'][:60], vertices=V, faces=F, rgba=RGBA, uv=UV, material=MAT, alpha=ALPHA, weights=W)
    path = folder/'mesh.json'
    path.write_text(json.dumps(j))
    info = dict(spec=spec, name=d['name'], models=d['models'], vertices=len(V), faces=len(F), materials=used,
                untextured_materials=missing, translucent_faces=sum(1 for x in ALPHA if x > 0),
                max_bones=max((len(w) for w in W), default=0))
    return path, info
