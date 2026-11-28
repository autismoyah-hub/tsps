"""fit_backing.py: which tiled backing (sprite, tile origin, transparency) reproduces a region of the sheet exactly,
drawn over the panel's own TRADEBACKING base. Measurement from the picture only."""
import sys, itertools
from pathlib import Path
import numpy as np
from PIL import Image

HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))
import ifcache as C, iface

iface.TSPS_BLEND = True
cache = C.Cache(C.CACHE_242)
img = np.asarray(Image.open(HERE / 'proof' / 'sheet-148-as-tsps-draws-it.png').convert('RGB')).astype(np.int64)
rgb = (img[..., 0] << 16) | (img[..., 1] << 8) | img[..., 2]
W, H = 400, 300


def draw(layers):
    r = iface.Raster(W, H, 0xff000000)
    for sid, ox, oy, t in layers:
        p = iface.Pixels(C.sprite_sheet(cache, sid))
        p.draw_widget(r, [ox, oy, W - ox, H - oy], 0, True, True, 256 - t)
    return r.px & 0xffffff


regions = {'A (light)': (10, 45, 130, 120), 'B (text box)': (10, 175, 25, 230), 'C (light)': (10, 245, 130, 290),
           'D (right)': (160, 45, 390, 290)}
base = (297, 1, 1, 0)
for name, (x0, y0, x1, y1) in regions.items():
    found = []
    target = rgb[y0:y1, x0:x1]
    if np.array_equal(draw([base])[y0:y1, x0:x1], target):
        found.append('panel base 297 from (1,1) alone')
    for sid in (297, 1040, 897):
        for ox, oy in itertools.product(range(0, 160, 1), range(0, 60, 1)):
            pass
    # tile origins worth trying: the box corners measured (0,30),(0,130),(0,232),(145,30) plus 0..6 px insets
    for sid in (297, 1040, 897):
        for (bx, by) in ((0, 30), (0, 130), (0, 232), (145, 30)):
            for d in range(0, 7):
                for t in list(range(0, 256, 25)) + [125, 150, 175, 200, 225, 240]:
                    out = draw([base, (sid, bx + d, by + d, t)])[y0:y1, x0:x1]
                    if np.array_equal(out, target):
                        found.append((sid, bx + d, by + d, t))
    print(name, found[:6])
