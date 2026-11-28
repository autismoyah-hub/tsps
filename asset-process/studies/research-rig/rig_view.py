"""A rig card: the bind-pose mesh in three orthographic views, every vertex coloured by its dominant bone, the bone tree
drawn over it with bone numbers.

usage: rig_view.py WORKDIR OUT.png [--model OURS.bin] [--scale S] [--errors SCORE.json] [--no-numbers]
--model   colour by another model file's weights (same skeleton);  --errors: colour by rig_wscore's per-vertex worst
displacement instead (black = 0, red = 3% of the creature's size or more).
"""
import sys, json, pathlib, colorsys
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
from PIL import Image, ImageDraw, ImageFont
import riglib as R
B, RM = R.B, R.RM
a = sys.argv; work = pathlib.Path(a[1]).resolve(); man = json.loads((work / 'manifest.json').read_text()); src = work / man['src']
def opt(n, d=None): return a[a.index(n) + 1] if n in a else d
sk = RM.Skeleton(B.decode_base((src / man['base_file']).read_bytes()))
m = B.decode_model(pathlib.Path(opt('--model') or (src / man['models'][0])).read_bytes())
V = np.array(m['vertices'], float); V[:, 1] *= -1; H = sk.head().copy(); H[:, 1] *= -1                  # y up
F = np.array(m['faces']); dom = np.array([max(w, key=lambda bw: bw[1])[0] if w else 0 for w in m['bone_weights']])
lo = np.minimum(V.min(0), H.min(0)) - 20; hi = np.maximum(V.max(0), H.max(0)) + 20; S = float(opt('--scale', 1.4))
def colour(b):
    r, g, bl = colorsys.hsv_to_rgb((b * 0.61803) % 1, 0.55 + 0.35 * ((b * 7) % 3) / 2, 0.95 - 0.3 * ((b * 5) % 4) / 3); return (int(r * 255), int(g * 255), int(bl * 255))
vc = [colour(int(b)) for b in dom]
if opt('--errors'):
    e = np.array(json.loads(pathlib.Path(opt('--errors')).read_text())['per_vertex']); big = float((V.max(0) - V.min(0)).max()); t = np.clip(e / (0.03 * big), 0, 1)
    vc = [(int(60 + 195 * x), int(60 * (1 - x)), int(60 * (1 - x))) for x in t]
views = [('side: x across is depth (front = left)', 2, 1), ('front (from -z)', 0, 1), ('top (front = up)', 0, 2)]
panels = []
try: font = ImageFont.truetype('segoeui.ttf', 11); big_font = ImageFont.truetype('segoeuib.ttf', 15)
except Exception: font = big_font = ImageFont.load_default()
for title, ax, ay in views:
    w, h = int((hi[ax] - lo[ax]) * S), int((hi[ay] - lo[ay]) * S); im = Image.new('RGB', (w, h + 22), (24, 22, 20)); d = ImageDraw.Draw(im)
    flip = ay == 1
    def P(p): return ((p[ax] - lo[ax]) * S, 22 + ((hi[ay] - p[ay]) if flip else (p[ay] - lo[ay])) * S)
    for f in F:
        pts = [P(V[i]) for i in f]; d.line(pts + [pts[0]], fill=(70, 66, 60), width=1)
    for i in range(len(V)):
        x, y = P(V[i]); d.rectangle([x - 1, y - 1, x + 1, y + 1], fill=vc[i])
    for i in range(sk.n):
        if sk.parent[i] >= 0: d.line([P(H[i]), P(H[sk.parent[i]])], fill=(235, 235, 235), width=1)
    for i in range(sk.n):
        x, y = P(H[i]); c = colour(i); d.ellipse([x - 3, y - 3, x + 3, y + 3], fill=c, outline=(255, 255, 255))
        if '--no-numbers' not in a: d.text((x + 4, y - 6), str(i), fill=(255, 255, 200), font=font)
    d.text((6, 3), title, fill=(255, 255, 255), font=big_font); panels.append(im)
W = sum(p.width for p in panels) + 10 * (len(panels) - 1); Hh = max(p.height for p in panels); out = Image.new('RGB', (W, Hh), (24, 22, 20)); x = 0
for p in panels: out.paste(p, (x, 0)); x += p.width + 10
pathlib.Path(a[2]).parent.mkdir(parents=True, exist_ok=True); out.save(a[2]); print(a[2], out.size)
