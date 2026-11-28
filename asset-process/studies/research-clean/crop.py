"""Close crops of a height band of several creatures side by side, lit; optional wire of the mesh edges.
usage: crop.py OUT.png ENTRY,ENTRY.. --band 0.55,0.9 [--yaw 20] [--size 500] [--wire] [--paint]
band = fractions of the height from the ground (0) to the top (1)."""
import sys, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
from PIL import Image, ImageDraw, ImageFont
import cleanlib as cl

def font(n):
    try: return ImageFont.truetype('segoeui.ttf', n)
    except Exception: return ImageFont.load_default()

def band_box(c, lo_f, hi_f, xfrac=1.0):
    skin = c['drawn'] & (c['A'] == 0)
    used = np.unique(c['F'][skin]); V = c['V'][used]; lo, hi = V.min(0), V.max(0); h = hi[1] - lo[1]
    y1 = hi[1] - lo_f * h; y0 = hi[1] - hi_f * h
    sel = V[(V[:, 1] >= y0) & (V[:, 1] <= y1)]
    b0, b1 = sel.min(0), sel.max(0); b0[1], b1[1] = y0, y1
    if xfrac < 1:
        cx = (b0[0] + b1[0]) / 2; hw = (b1[0] - b0[0]) / 2 * xfrac; b0[0], b1[0] = cx - hw, cx + hw
    return b0, b1

def draw_wire(img, c, yaw, pitch, box, size, pad=8, col=(0, 0, 0), faces=None):
    """edges of drawn faces over a picture made by cl.view with the same arguments (front-facing ones only)"""
    r, down, fwd = cl.camera(yaw, pitch); V = c['V']
    P = np.stack([V @ r, V @ down, V @ fwd], 1)
    corners = np.array([[box[(k >> 0) & 1][0], box[(k >> 1) & 1][1], box[(k >> 2) & 1][2]] for k in range(8)], float)
    Pc = np.stack([corners @ r, corners @ down, corners @ fwd], 1); lo, hi = Pc.min(0), Pc.max(0)
    s = (size - 2 * pad) / max(hi[0] - lo[0], hi[1] - lo[1], 1)
    im = Image.fromarray(img); d = ImageDraw.Draw(im, 'RGBA')
    for i in (np.nonzero(c['drawn'])[0] if faces is None else faces):
        if c['unit'][i] @ fwd > 0: continue
        pts = [((P[v, 0] - lo[0]) * s + pad, (P[v, 1] - lo[1]) * s + pad) for v in c['F'][i]]
        d.line(pts + [pts[0]], fill=col + (90,), width=1)
    return np.array(im)

if __name__ == '__main__':
    a = sys.argv[1:]
    out = a[0]; entries = a[1].split(',')
    lo_f, hi_f = [float(x) for x in a[a.index('--band') + 1].split(',')]
    yaws = [float(x) for x in a[a.index('--yaw') + 1].split(',')] if '--yaw' in a else [20]
    size = int(a[a.index('--size') + 1]) if '--size' in a else 500
    xfrac = float(a[a.index('--xfrac') + 1]) if '--xfrac' in a else 1.0
    cols = []
    for e in entries:
        c = cl.load(e); cells = []
        for y in yaws:
            box = band_box(c, lo_f, hi_f, xfrac)
            img, ids = cl.view(c, y, 8, size=size, box=box, light='--paint' not in a)
            if '--wire' in a: img = draw_wire(img, c, y, 8, box, size)
            canvas = np.zeros((size, size, 3), np.uint8); canvas[:] = cl.BG
            h, w = min(img.shape[0], size), min(img.shape[1], size); canvas[:h, :w] = img[:h, :w]
            im = Image.fromarray(canvas); ImageDraw.Draw(im).text((4, 2), f"{c['name']}", fill=(240, 230, 160), font=font(14)); cells.append(np.array(im))
        cols.append(np.concatenate(cells, 0))
    Image.fromarray(np.concatenate(cols, 1)).save(out); print(out)
