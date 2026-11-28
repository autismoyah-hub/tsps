"""Score a rebuild against the real object it copies. usage: compare.py REAL_OBJECT_ID OURS.bin  |  compare.py item:ID OURS.bin (worn model)

Measures (each of ours beside the real figure): size, counts, welding, shading, colour (the surface's share at each
hue/saturation/lightness after the definition's recolours), and outline overlap in six orthographic views.
The Teacher runs this; authors of a held-out rebuild do not.
"""
import sys, json, math, collections, pathlib
sys.dont_write_bytecode = True
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parents[2] / 'tools'))
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from objlight import decode
import osrscache as o, scene
from measure import area
import numpy as np
from PIL import Image, ImageDraw


def recolours(defb):
    definition = decode(defb)
    out = dict(definition['recolor'])
    out.update({('scale', axis): value for axis, value in definition['scale'].items()})
    return out


def assembled(parts, faithful=False):
    painted = any(any(m['alpha']) or any(t>=0 for t in m['face_textures']) for m in parts)
    return o.combine_for_render(parts) if faithful or painted else o.merge(parts)


def apply_textures(m, definition):
    for old,new in definition.get('retextures') or []:
        m['face_textures'] = [new if t==old else t for t in m['face_textures']]


def real_model(oid, shape=None, faithful=False):
    """A real object's mesh with its definition's recolours and scale applied. The object is found in whichever
    archive shape folder holds it (scene.object_json); shape=N picks the model the object lists for that placement
    shape (default: the folder's own shape). The result also carries `depth_bias` (per-face depth offsets) and
    `object` (the archive JSON with shape_folder / shape_used / shapes_listed / shape_model_ids)."""
    j = scene.object_json(oid, shape)
    parts = [o.model(i) for i in j['shape_model_ids']]
    m = assembled(parts, faithful)
    apply_textures(m,j)
    m['depth_bias'] = [b for p in parts for b in p['depth_bias']]
    m['object'] = j
    rc = recolours(scene.object_def(oid))
    m['colors'] = [rc.get(c, c) for c in m['colors']]
    sc = [rc.get(('scale', i), 128) / 128 for i in range(3)]
    m['vertices'] = [(p[0] * sc[0], p[1] * sc[1], p[2] * sc[2]) for p in m['vertices']]
    return m


def real_item(iid):
    """A real item's WORN model (male), with the recolours listed in its archive entry applied."""
    j = next(json.loads(p.read_text()) for p in (scene.ARCH / 'items').glob(f'*/{iid}.json'))
    ids = [k for k in j['wear_models'][0] if k not in (-1, None)]
    m = assembled([o.model(k) for k in ids])
    apply_textures(m,j)
    rc = {int(a): int(b) for a, b in (j.get('recolors') or [])}
    m['colors'] = [rc.get(c, c) for c in m['colors']]
    return m


def colour_shares(m):
    ar = collections.Counter()
    for f, c in zip(m['faces'], m['colors']): ar[o.hsl_split(c)] += area(m['vertices'], f)
    t = sum(ar.values()) or 1
    return {k: v / t for k, v in ar.items()}


def colour_distance(a, b):
    """Earth-mover-like: each unit of our surface is matched to the nearest real colour still unfilled.
    Distance between colours: hue steps (wrapped) + saturation steps + lightness steps / 4. 0 is identical."""
    def d(x, y):
        dh = min(abs(x[0] - y[0]), 64 - abs(x[0] - y[0]))
        w = min(x[1], y[1]) / 7 + 0.15            # hue matters little when a colour is nearly grey
        return dh * w + abs(x[1] - y[1]) + abs(x[2] - y[2]) / 4
    need = dict(b); cost = 0.0
    for k, share in sorted(a.items(), key=lambda kv: -kv[1]):
        left = share
        for t in sorted(need, key=lambda t: d(k, t)):
            if left <= 1e-9: break
            take = min(left, need[t])
            if take <= 0: continue
            cost += take * d(k, t); need[t] -= take; left -= take
    return cost


VIEWS = {'front': (0, 1, 1, -1), 'side': (2, 1, 1, -1), 'top': (0, 2, 1, 1)}


def material_info(model, population):
    """Asset face counts and revision-239 archive population using each channel."""
    textures = collections.Counter(t for t in model['face_textures'] if t >= 0)
    alpha = collections.Counter(a for a in model['alpha'] if a)
    bands = json.loads((pathlib.Path(__file__).with_name('material_bands.json')).read_text())
    return {'textured faces by ID':dict(textures), 'faces by nonzero transparency':dict(alpha),
            'population':population, 'real':bands.get(population)}


def silhouette(m, view, lo, scale, size):
    ax, ay, sx, sy = VIEWS[view]
    im = Image.new('1', (size, size), 0); d = ImageDraw.Draw(im)
    v = m['vertices']
    for f in m['faces']:
        pts = [((v[i][ax] - lo[ax]) * scale + 8, (v[i][ay] - lo[ay]) * scale + 8) for i in f]
        d.polygon(pts, fill=1)
    return np.array(im)


def report(oid, ours_path, kind='object', shape=None):
    # shape: placement shape whose model(s) the real object is taken as (None = its archive folder's own shape)
    R = real_item(oid) if kind == 'item' else real_model(oid, shape); U = o.decode_model(pathlib.Path(ours_path).read_bytes())
    population = (next((scene.ARCH/'items').glob(f'*/{oid}.json')).parent.name if kind=='item'
                  else f"shape-{R['object']['shape_used']}")
    print('INFO materials', json.dumps(material_info(U, population)))
    print('INFO reference faces', dict(collections.Counter(R['face_textures'])), dict(collections.Counter(R['alpha'])))
    def ext(m):
        v = m['vertices']; return [(min(p[i] for p in v), max(p[i] for p in v)) for i in range(3)]
    er, eu = ext(R), ext(U)
    rows = []
    rows.append(('size x, height, z', [round(b - a) for a, b in er], [round(b - a) for a, b in eu]))
    rows.append(('vertices', len(R['vertices']), len(U['vertices'])))
    rows.append(('faces', len(R['faces']), len(U['faces'])))
    dup = lambda m: sum(n for n in collections.Counter(m['vertices']).values() if n > 1)   # after scaling, still exact for the real one
    rows.append(('vertices on a shared position', dup(R), dup(U)))
    rows.append(('flat-shaded faces', sum(1 for r in R['render_types'] if r), sum(1 for r in U['render_types'] if r)))
    cr, cu = colour_shares(R), colour_shares(U)
    rows.append(('colours', len(cr), len(cu)))
    for name, a, b in rows: print(f'{name:32s} real {a}   ours {b}')
    print('colour shares (h,s,l: % of surface)')
    print('  real:', ', '.join(f'{k}:{100*v:.0f}' for k, v in sorted(cr.items(), key=lambda kv: -kv[1])[:10]))
    print('  ours:', ', '.join(f'{k}:{100*v:.0f}' for k, v in sorted(cu.items(), key=lambda kv: -kv[1])[:10]))
    print(f'colour distance (0 = identical; about 1 = one lightness step of 4 over the whole surface): {colour_distance(cu, cr):.2f}')
    # outlines: both centred on their own bounds in x and z, both standing on their base
    def centred(m, e):
        cx = (e[0][0] + e[0][1]) / 2; cz = (e[2][0] + e[2][1]) / 2; top = e[1][0]
        return dict(m, vertices=[(p[0] - cx, p[1] - e[1][1], p[2] - cz) for p in m['vertices']])
    # a worn item is authored in body space, so its position matters: compare it where it stands
    Rc, Uc = (R, U) if kind == 'item' else (centred(R, er), centred(U, eu))
    allv = Rc['vertices'] + Uc['vertices']
    lo = [min(p[i] for p in allv) for i in range(3)]; hi = [max(p[i] for p in allv) for i in range(3)]
    size = 400; scale = (size - 16) / max(h - l for l, h in zip(lo, hi))
    ious = {}
    for view in VIEWS:
        a = silhouette(Rc, view, lo, scale, size); b = silhouette(Uc, view, lo, scale, size)
        ious[view] = (a & b).sum() / max((a | b).sum(), 1)
    print('outline overlap (1 = identical):', ', '.join(f'{k} {v:.3f}' for k, v in ious.items()))
    return rows, ious


if __name__ == '__main__':
    a = sys.argv[1]
    if a.startswith('item:'):
        report(int(a[5:]), sys.argv[2], 'item')
    else:
        report(int(a), sys.argv[2])
