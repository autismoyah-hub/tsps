"""Adapt build-951 presentations to the tools-hd-gate sheet and gate (revision tag 951).

The RS3 original is converted to OSRS axes and units: (x, y, z) / 4 with y negated, winding reversed,
positions rounded to whole game units and welded. Per-vertex HSL -> face colour of the first corner;
per-vertex alpha -> face transparency 255 - mean(alpha); group material -> face texture (RS3 material id);
RS3 UVs are carried as per-corner projected coordinates. The material swatch is the material's diffuse
image (first texture reference, archive 53), resized to 128 x 128 for the classic rasterizer.
"""
import sys, json
from pathlib import Path
from functools import lru_cache
ROOT = Path(__file__).resolve().parent
GATE = ROOT.parent/'tools-hd-gate'
sys.dont_write_bytecode = True
sys.path[:0] = [str(ROOT), str(GATE), str(GATE.parent/'tools-hd-era')]
import numpy as np
from PIL import Image
import hdasset
import rs3mesh

REV = 951
KINDS = ['model', 'npc', 'item-ground', 'item-worn', 'item-female'] + [f'npc-part{k}' for k in range(1, 9)]
_orig_cache = hdasset.cache

class RS3Handle:
    rev = REV

def cache(rev):
    return RS3Handle() if rev == REV else _orig_cache(rev)

@lru_cache(None)
def _swatch(material):
    p = rs3mesh.diffuse(material)
    if p is None:
        return Image.new('RGBA', (128, 128), (128, 128, 128, 255)), 'no diffuse image'
    # Diffuse alpha channels carry non-opacity data in many materials; opacity comes from vertex alpha.
    return Image.open(p).convert('RGB').convert('RGBA').resize((128, 128), Image.Resampling.LANCZOS), None

_orig_preview = hdasset.preview
def preview(c, t):
    if getattr(c, 'rev', None) == REV:
        return _swatch(t)
    return _orig_preview(c, t)

def to_osrs(m):
    """One decoded RS3 model (after definition recolour/retexture) as an OSRS-style dict."""
    p = np.asarray(m['positions'], float)
    conv = np.rint(np.stack([p[:, 0]/4, -p[:, 1]/4, p[:, 2]/4], 1)).astype(int)
    uniq, inverse = np.unique(conv, axis=0, return_inverse=True)
    inverse = inverse.reshape(-1)
    alpha = m.get('alpha', np.full(len(p), 255))
    faces, colors, tex, trans, uv = [], [], [], [], []
    for g in m['groups']:
        t = g['triangles'][:, ::-1]
        faces.append(inverse[t]); colors += [int(m['colours'][a]) for a in t[:, 0]]
        tex += [int(g['material'])]*len(t)
        trans.append(255 - np.rint(alpha[t].mean(1)).astype(int))
        uv.append(np.asarray(m['uv'], np.float32)[t])
    faces = np.concatenate(faces).tolist() if faces else []
    n = len(faces)
    textured = [1 if x >= 0 else 0 for x in tex]
    out = dict(vertices=[tuple(map(int, v)) for v in uniq], faces=[tuple(f) for f in faces], colors=colors,
               render_types=[0]*n, priorities=[0]*n, alpha=np.concatenate(trans).tolist() if trans else [],
               face_textures=tex, texture_coords=[0 if k else -1 for k in textured],
               texture_types=[1] if any(textured) else [], texture_triangles=[(0, 0, 0)] if any(textured) else [],
               complex_sections=(['00'*6, '00'*6, '00'*2, '00', '00'*2] if any(textured) else ['']*5),
               depth_bias=[0]*n, face_skins=[0]*n, vertex_skins=None, has_face_skin=False,
               has_bone_weights=False, bone_weights=None, uniform_priority=0, has_alpha=any(x for x in np.concatenate(trans)) if trans else False,
               format='extended', has_render=False)
    out['uv'] = np.concatenate(uv) if uv else np.zeros((0, 3, 2), np.float32)
    return out

def target(rev, kind, number):
    if rev != REV: return _orig_target(rev, kind, number)
    part = None
    if kind.startswith('npc-part'):
        part = int(kind[8:]) - 1
        kind_ = 'npc'
    else:
        kind_ = kind
    spec = {'item-ground': 'item', 'model': 'model'}.get(kind_, kind_) + f':{number}'
    d, ps = rs3mesh.parts(spec)
    if part is not None:
        # An NPC body over the 242 single-model stream limits is scored one definition model at a time.
        d = dict(d, models=[d['models'][part]]); ps = [ps[part]]
    if not ps: raise ValueError(f'No {kind} model for {number}')
    parts = [to_osrs(m) for m in ps]
    return dict(name=d['name'], models=d['models'], recolour=d['recolour'], retexture=d['retexture'], presentation=kind), parts

def conversion(rev, d, parts):
    """Ceiling: the converted original in our format, with nearest installed 242 textures and face-local UVs."""
    if rev != REV: return _orig_conversion(rev, d, parts)
    out, mapping = [], {}
    for p in parts:
        m = {k: (list(v) if isinstance(v, list) else v) for k, v in p.items() if k != 'uv'}
        for t in set(m['face_textures'])-{-1}:
            if t not in mapping:
                sig = hdasset.signature(_swatch(t)[0])
                cat = hdasset.catalogue()
                mapping[t] = min(cat, key=lambda k: (float(np.abs(sig-cat[k]).mean()), k))
        m['face_textures'] = [mapping.get(t, t) for t in m['face_textures']]
        m.update(texture_coords=[-1]*len(m['faces']), texture_types=[], texture_triangles=[], complex_sections=['']*5)
        out.append(m)
    return out, mapping

_orig_target, _orig_conversion = hdasset.target, hdasset.conversion

def install(*modules):
    """Route revision 951 through this adapter in hdasset and the given importers of `from hdasset import *`."""
    for mod in (hdasset,)+modules:
        for name, fn in (('cache', cache), ('preview', preview), ('target', target), ('conversion', conversion)):
            if hasattr(mod, name): setattr(mod, name, fn)
