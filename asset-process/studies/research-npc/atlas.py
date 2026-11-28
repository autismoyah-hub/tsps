"""Label atlas of real NPCs on one frame base: where each vertex label sits, per creature.

usage: atlas.py OUTDIR NPC_ID [NPC_ID...]
For each NPC writes OUTDIR/npc-ID.txt (the assembled rest mesh, from `asset_npc merged`) and OUTDIR/atlas-ID.png
(side view looking along +x, and top view; every vertex in the colour of its label with the label number at the
label's centroid; grid every 20 units), and prints one table: label -> vertex count and centroid (x, height, z)
in each creature. Cache coordinates: x is the creature's left-right, z is its length; which end is the head is
read off the picture (the head is where labels 8 / 69 / 92 / 42 sit on skeleton 1491).
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, subprocess, pathlib, colorsys, collections
from PIL import Image, ImageDraw, ImageFont
EXE = _AP.fix(r'<CLIENT_BIN>\asset_npc.exe')
CACHE = _AP.fix(r'<OSRS_CACHE_239>')
def read(path):
    V, L, F = [], [], []
    for line in pathlib.Path(path).read_text().splitlines():
        p = line.split()
        if p and p[0] == 'v': V.append((int(p[1]), -int(p[2]), int(p[3]))); L.append(int(p[4]))
        elif p and p[0] == 'f': F.append((int(p[1]), int(p[2]), int(p[3]), int(p[4])))
    return V, L, F
def colour(l):
    r, g, b = colorsys.hsv_to_rgb((l * 0.381966) % 1.0, 0.75, 1.0); return int(r * 255), int(g * 255), int(b * 255)
def picture(V, L, F, out, s=4):
    xs = [v[0] for v in V]; ys = [v[1] for v in V]; zs = [v[2] for v in V]
    pad = 20; w = (max(zs) - min(zs) + 2 * pad) * s; h1 = (max(ys) - min(ys) + 2 * pad) * s; h2 = (max(xs) - min(xs) + 2 * pad) * s
    im = Image.new('RGB', (w, h1 + h2), (24, 24, 28)); d = ImageDraw.Draw(im)
    try: font = ImageFont.truetype('segoeuib.ttf', 15)
    except Exception: font = ImageFont.load_default()
    side = lambda v: ((v[2] - min(zs) + pad) * s, (max(ys) - v[1] + pad) * s)
    top = lambda v: ((v[2] - min(zs) + pad) * s, h1 + (v[0] - min(xs) + pad) * s)
    for proj in (side, top):
        for g in range(-400, 400, 20):
            a = proj((0, g, g)); d.line([(a[0], 0 if proj is side else h1), (a[0], h1 if proj is side else h1 + h2)], fill=(40, 40, 46))
        for f in F:
            pts = [proj(V[i]) for i in f[:3]]; d.line(pts + [pts[0]], fill=(70, 70, 78))
        for v, l in zip(V, L): p = proj(v); d.rectangle([p[0] - 2, p[1] - 2, p[0] + 2, p[1] + 2], fill=colour(l))
        by = collections.defaultdict(list)
        for v, l in zip(V, L): by[l].append(v)
        for l, vs in by.items():
            c = tuple(sum(v[i] for v in vs) / len(vs) for i in range(3)); p = proj(c)
            d.text((p[0] + 3, p[1] - 8), str(l), fill=colour(l), font=font, stroke_width=2, stroke_fill=(0, 0, 0))
    d.text((6, 4), 'side (z across, height up)', fill=(220, 220, 220), font=font); d.text((6, h1 + 4), 'top (z across, x down)', fill=(220, 220, 220), font=font)
    im.save(out)
if __name__ == '__main__':
    out = pathlib.Path(sys.argv[1]); out.mkdir(parents=True, exist_ok=True); table = {}; ids = sys.argv[2:]
    for i in ids:
        spec = i if ':' in i else f'npc:{i}'; name = i.replace(':', '-').replace('/', '_').replace('\\', '_')[-40:]
        t = out / f'npc-{name}.txt'
        r = subprocess.run([EXE, CACHE, 'merged', spec, str(t)], capture_output=True, text=True)
        if r.returncode: raise SystemExit(r.stdout + r.stderr)
        V, L, F = read(t); picture(V, L, F, out / f'atlas-{name}.png')
        by = collections.defaultdict(list)
        for v, l in zip(V, L): by[l].append(v)
        for l, vs in by.items(): table.setdefault(l, {})[i] = (len(vs), tuple(round(sum(v[k] for v in vs) / len(vs)) for k in range(3)))
        print(r.stdout.strip(), 'x', min(v[0] for v in V), max(v[0] for v in V), 'height', min(v[1] for v in V), max(v[1] for v in V), 'z', min(v[2] for v in V), max(v[2] for v in V))
    print('label ' + ' '.join(f'{i:>22}' for i in ids))
    for l in sorted(table): print(f'{l:5d} ' + ' '.join(f'{str(table[l].get(i, "")):>22}' for i in ids))
