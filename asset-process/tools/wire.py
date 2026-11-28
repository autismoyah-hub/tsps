"""Wireframe study sheet of a real object: how its surface is split, toned and layered.

usage: wire.py REAL_OBJECT_ID OUT.png [--shape N]
Four views (front, right side, top, three-quarter). Each face is filled with its own colour (unlit), every
edge is drawn, flat-shaded faces are outlined in red, and a second row colours faces by draw priority.
When the model has per-face DEPTH OFFSETS a third row is added, colouring every face by its offset (grey = none,
yellow to red = rising positive offsets, blue = negative), and the histogram (offset: faces) is printed and
written under the sheet. On scenery a positive offset is how a face lying in another's plane is made to show
(the client subtracts it from the face's view depth, so the face wins against anything up to that many units in
front of it): inlays, mortar lines, painted patterns. In that row faces are layered the same way, so an offset
face shows over the surface it lies in. A model with no offsets gives the two-row sheet as before.
The object is looked up in whichever archive folder holds it (scene.object_json): shape-10 ordinary objects,
shape-0 / 3 / 9 walls and corners, shape-4 wall decorations, shape-22 floors and ground decorations. A wall
lists one model per placement shape; the folder's own shape is drawn unless --shape N chooses another (the
tool prints which shapes exist). The model is drawn as stored, before the rotation or mirroring a placement
applies: a shape-0 wall stands on the west edge of its tile (x near -64).
For the author pack: use on real NEIGHBOURS of what is being built, never on a held-out test piece.
"""
import sys, math, pathlib, colorsys, collections
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import osrscache as o, compare
from PIL import Image, ImageDraw

args = sys.argv[1:]
shape = None
if '--shape' in args:
    k = args.index('--shape'); shape = int(args[k + 1]); del args[k:k + 2]
oid, out = int(args[0]), args[1]
m = compare.real_model(oid, shape)
J = m['object']
V = [(x, -y, z) for x, y, z in m['vertices']]          # y up
F = m['faces']
lo = [min(p[i] for p in V) for i in range(3)]; hi = [max(p[i] for p in V) for i in range(3)]
span = max(h - l for l, h in zip(lo, hi)); S = 520; sc = (S - 60) / (span * 1.25)
c3 = [(l + h) / 2 for l, h in zip(lo, hi)]
a, e = math.radians(35), math.radians(30)


def proj(view, p):
    x, y, z = (p[i] - c3[i] for i in range(3))
    if view == 'front': u, v, d = x, y, z                 # looking from -z
    elif view == 'side': u, v, d = -z, y, -x              # looking from +x
    elif view == 'top': u, v, d = x, -z, y                # looking down, front at the bottom
    else:
        xr = x * math.cos(a) + z * math.sin(a); zr = -x * math.sin(a) + z * math.cos(a)
        u, v, d = xr, y * math.cos(e) - zr * math.sin(e), zr * math.cos(e) + y * math.sin(e)
        d = d
    return S / 2 + u * sc, S / 2 - v * sc, d


def rgb(c):
    h, s, l = o.hsl_split(c)
    r, g, b = colorsys.hls_to_rgb(h / 64 + 1 / 128, min(1, l / 127 * 1.25), s / 8 + 1 / 16)
    return int(r * 255), int(g * 255), int(b * 255)


PR = {0: (70, 110, 200), 1: (80, 170, 90), 2: (210, 160, 60), 3: (200, 90, 90), 4: (150, 110, 190)}
DEPTH = m.get('depth_bias') or [0] * len(F)
offsets = collections.Counter(DEPTH)
has_depth = any(DEPTH)
POS = [(235, 215, 80), (240, 170, 50), (235, 115, 45), (220, 60, 50), (200, 40, 120), (150, 40, 160)]
NEG = [(120, 190, 235), (70, 130, 220), (50, 80, 190)]
DC = {0: (70, 76, 84)}
for k, v in enumerate(sorted(x for x in offsets if x > 0)): DC[v] = POS[min(k, len(POS) - 1)]
for k, v in enumerate(sorted((x for x in offsets if x < 0), reverse=True)): DC[v] = NEG[min(k, len(NEG) - 1)]
views = ['front', 'side', 'top', 'threeq']
modes = ('tone', 'priority') + (('depth offset',) if has_depth else ())
R = len(modes)
sheet = Image.new('RGB', (S * 4, S * R + 70 + (22 if has_depth else 0)), (22, 27, 34)); D = ImageDraw.Draw(sheet)
for row, mode in enumerate(modes):
    for col, view in enumerate(views):
        im = Image.new('RGB', (S, S), (30, 37, 46)); d = ImageDraw.Draw(im)
        order = []
        for i, f in enumerate(F):
            pts = [proj(view, V[k]) for k in f]
            # in 'top' the nearer face has larger height; elsewhere larger d is farther for front (z toward back)
            depth = sum(p[2] for p in pts) / 3
            far = depth if view in ('front', 'threeq') else -depth
            if mode == 'depth offset': far -= DEPTH[i]          # as the client: the offset brings the face nearer
            order.append((far, i, pts))
        order.sort(reverse=True)
        for _, i, pts in order:
            xy = [(p[0], p[1]) for p in pts]
            ar = (xy[1][0] - xy[0][0]) * (xy[2][1] - xy[0][1]) - (xy[2][0] - xy[0][0]) * (xy[1][1] - xy[0][1])
            fill = rgb(m['colors'][i]) if mode == 'tone' else (PR.get(m['priorities'][i], (120, 120, 120)) if mode == 'priority' else DC[DEPTH[i]])
            flat = m['render_types'][i] == 1
            d.polygon(xy, fill=fill, outline=(220, 40, 40) if (flat and mode == 'tone') else (15, 15, 15))
        d.text((8, 6), f'{view}  ({mode})', fill=(235, 235, 235))
        sheet.paste(im, (col * S, row * S))
tones = collections.Counter(o.hsl_split(c) for c in m['colors'])
which = '' if (J['shape_folder'] == 10 and J['shape_used'] == 10) else f' (shape {J["shape_used"]}, archive folder shape-{J["shape_folder"]})'
txt = (f'real object {oid}{which}: {len(V)} vertices, {len(F)} faces, {sum(1 for r in m["render_types"] if r == 1)} flat (red outline); '
       f'size {round(hi[0]-lo[0])} x {round(hi[1]-lo[1])} x {round(hi[2]-lo[2])}; colours (h,s,l: faces) '
       + ', '.join(f'{k}:{n}' for k, n in tones.most_common(8)))
D.text((10, S * R + 8), txt, fill=(230, 230, 230))
D.text((10, S * R + 30), 'priority colours: 0 blue, 1 green, 2 amber, 3 red, 4 violet (lower is drawn first). Fills are the painted tone, before lighting.', fill=(170, 180, 190))
hist = ', '.join(f'{k}: {n}' for k, n in sorted(offsets.items()))
if has_depth:
    x = 10
    D.text((x, S * R + 52), 'depth offsets (offset: faces)', fill=(170, 180, 190)); x += 200
    for k, n in sorted(offsets.items()):
        D.rectangle([x, S * R + 53, x + 12, S * R + 65], fill=DC[k], outline=(15, 15, 15))
        D.text((x + 17, S * R + 52), f'{k}: {n}', fill=(230, 230, 230)); x += 30 + 8 * len(f'{k}: {n}')
    D.text((x + 10, S * R + 52), 'a positive offset draws the face as if that many units nearer: it shows over the surface it lies in', fill=(170, 180, 190))
sheet.save(out); print(out)
print(f'object {oid} {J.get("name")!r}: archive folder shape-{J["shape_folder"]}, drawn shape {J["shape_used"]} model(s) {J["shape_model_ids"]}'
      + (f'; lists models for shapes {J["shapes_listed"]}' if J['shapes_listed'] else '; untyped models')
      + f'; {len(V)} vertices, {len(F)} faces')
print(f'depth offsets (offset: faces) {{{hist}}}' + ('' if has_depth else '  -> none: no depth-offset row'))
