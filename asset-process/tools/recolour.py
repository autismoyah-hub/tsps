"""Repaint one of our models: move every face of one hue and saturation to another, keeping lightness.

usage: recolour.py IN.bin OUT.bin H,S:H2,S2[:DL] [H,S:H2,S2[:DL] ...]
Each rule moves faces painted hue H, saturation S to hue H2, saturation S2; DL (optional, may be negative) is added
to their lightness. Geometry, labels, priorities and shading are kept. For trying a palette change over a finished
piece (for example the house steel at another hue) without rebuilding it. Prints how many faces each rule moved.
"""
import sys, pathlib, collections
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import osrscache as o, author
from author import native_codec

src, out = sys.argv[1], sys.argv[2]
rules = {}
for r in sys.argv[3:]:
    p = r.split(':'); a = tuple(map(int, p[0].split(','))); b = tuple(map(int, p[1].split(',')))
    rules[a] = (b[0], b[1], int(p[2]) if len(p) > 2 else 0)
m = o.decode_model(pathlib.Path(src).read_bytes())
moved = collections.Counter(); cols = []
for c in m['colors']:
    h, s, l = o.hsl_split(c)
    if (h, s) in rules:
        h2, s2, dl = rules[(h, s)]; moved[(h, s)] += 1
        c = (h2 << 10) | (s2 << 7) | max(0, min(127, l + dl))
    cols.append(c)
g = dict(vertices=[list(p) for p in m['vertices']], faces=m['faces'], colors=cols)
if m.get('vertex_skins'): g['vertex_skins'] = m['vertex_skins']
if m.get('face_skins'): g['face_skins'] = m['face_skins']
if len(set(m['priorities'])) > 1: g['priorities'] = m['priorities']
else: g['priority'] = m['priorities'][0]
if any(m['render_types']): g['render_types'] = m['render_types']
if m.get('alpha') and any(m['alpha']): g['alpha'] = m['alpha']
pathlib.Path(out).write_bytes(native_codec.model(g))
print(out, 'moved', {f'{k[0]},{k[1]}': n for k, n in moved.items()}, 'of', len(cols), 'faces')
