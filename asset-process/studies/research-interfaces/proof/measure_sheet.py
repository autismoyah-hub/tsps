"""measure_sheet.py SHEET.png: measure a panel picture the way an author would, using only the picture and the
cache's sprites (never the panel's component list). Finds where the steel-border strips, corners and backgrounds
sit (exact pixel matches of the sprites' visible pixels), and the bounding box and colour of each text run."""
import sys, json
from pathlib import Path
import numpy as np
from PIL import Image

HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))
import ifcache as C, iface

cache = C.Cache(C.CACHE_242)
img = np.asarray(Image.open(sys.argv[1]).convert('RGB')).astype(np.int64)
H, W, _ = img.shape
rgb = (img[..., 0] << 16) | (img[..., 1] << 8) | img[..., 2]


def sprite(sid):
    p = iface.Pixels(C.sprite_sheet(cache, sid))
    return p


def find(sid, step=1):
    """Every placement (x, y) of the sprite's canvas where all its opaque pixels equal the picture."""
    p = sprite(sid)
    ox, oy = p.offset; w, h = p.size
    m = p.p != 0
    hits = []
    for y in range(-oy, H - oy - h + 1):
        for x in range(-ox, W - ox - w + 1):
            win = rgb[y + oy:y + oy + h, x + ox:x + ox + w]
            if np.array_equal(win[m], p.p[m] & 0xffffff):
                hits.append((x, y))
    return hits


out = {}
for sid in (310, 311, 312, 313, 535, 172, 315, 314, 173, 2546):
    hits = find(sid)
    out[sid] = hits[:400]
    print(sid, len(hits), hits[:12])
# text runs: pixels far brighter than any background tone, grouped by rows
bright = (img.max(-1) > 200) | ((img[..., 0] > 180) & (img[..., 2] < 80))
rows = np.nonzero(bright.any(1))[0]
runs = []
if len(rows):
    s = rows[0]; prev = rows[0]
    for r in list(rows[1:]) + [10 ** 9]:
        if r != prev + 1:
            ys = slice(s, prev + 1)
            xs = np.nonzero(bright[ys].any(0))[0]
            cols = {}
            for y in range(s, prev + 1):
                for x in np.nonzero(bright[y])[0]:
                    cols['%06x' % rgb[y, x]] = cols.get('%06x' % rgb[y, x], 0) + 1
            runs.append(dict(y0=int(s), y1=int(prev), x0=int(xs.min()), x1=int(xs.max()), colours=sorted(cols.items(), key=lambda kv: -kv[1])[:3]))
            s = r
        prev = r
out['text_runs'] = runs
for r in runs:
    print('text', r)
Path(sys.argv[2] if len(sys.argv) > 2 else 'sheet-measure.json').write_text(json.dumps(out, indent=0, default=int))
