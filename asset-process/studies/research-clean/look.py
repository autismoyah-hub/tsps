"""Look at creatures: lit whole-body and head close-ups, by the own renderer.
usage: look.py OUT.png ENTRY[,ENTRY..] [--head] [--yaw a,b,c] [--size N] [--wire]"""
import sys, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
from PIL import Image, ImageDraw, ImageFont
import cleanlib as cl

def font(n):
    try: return ImageFont.truetype('segoeui.ttf', n)
    except Exception: return ImageFont.load_default()

def head_box(c, frac=0.22):
    """The top `frac` of the drawn figure by height, around the highest mass (a rough head box)."""
    used = np.unique(c['F'][c['drawn'] & (c['A'] == 0)])
    V = c['V'][used]; lo, hi = V.min(0), V.max(0)
    top = lo[1] + frac * (hi[1] - lo[1])
    sel = V[V[:, 1] <= top]
    return sel.min(0), sel.max(0)

def wire(img, c, yaw, pitch, box, size):
    return img

if __name__ == '__main__':
    a = sys.argv[1:]
    out = a[0]; entries = a[1].split(',')
    yaws = [float(x) for x in a[a.index('--yaw') + 1].split(',')] if '--yaw' in a else [20, 90, 200]
    size = int(a[a.index('--size') + 1]) if '--size' in a else 360
    frac = float(a[a.index('--frac') + 1]) if '--frac' in a else 0.22
    rows = []
    for e in entries:
        c = cl.load(e)
        cells = []
        for y in yaws:
            box = head_box(c, frac) if '--head' in a else None
            img, ids = cl.view(c, y, 8, size=size, box=box)
            canvas = np.zeros((size, size, 3), np.uint8); canvas[:] = cl.BG
            h, w = min(img.shape[0], size), min(img.shape[1], size)
            canvas[:h, :w] = img[:h, :w]; cells.append(canvas)
        row = Image.fromarray(np.concatenate(cells, 1)); ImageDraw.Draw(row).text((4, 2), f"{c['name']}  faces {int(c['drawn'].sum())}", fill=(240, 230, 160), font=font(14))
        rows.append(row)
    W = max(r.width for r in rows); H = sum(r.height for r in rows)
    board = Image.new('RGB', (W, H), cl.BG); y = 0
    for r in rows: board.paste(r, (0, y)); y += r.height
    board.save(out); print(out, board.size)
