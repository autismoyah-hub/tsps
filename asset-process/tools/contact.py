"""Contact sheet of archive thumbnails for a name pattern. usage: contact.py KIND SUB REGEX N OUT [minfaces]"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, json, re, pathlib
from PIL import Image, ImageDraw
ARCH = pathlib.Path(_AP.fix(r'<REFERENCE_ARCHIVE>'))
kind, sub, rx, n, out = sys.argv[1:6]
minf = int(sys.argv[6]) if len(sys.argv) > 6 else 0
d = ARCH / kind / sub
ids = sorted((int(p.stem) for p in d.glob('*.json')), reverse=True)
sel, seen = [], set()
for i in ids:
    j = json.loads((d / f'{i}.json').read_text())
    key = tuple(j.get('model_ids') or [])
    if not key or key in seen or not re.search(rx, j.get('name') or '', re.I) or (j.get('faces') or 0) < minf:
        continue
    seen.add(key); sel.append(j)
    if len(sel) >= int(n):
        break
cols = int(sys.argv[7]) if len(sys.argv) > 7 else 8
tw, th = (1600 // cols, 1600 // cols)
sheet = Image.new('RGB', (cols * tw, ((len(sel) + cols - 1) // cols) * (th + 28)), (20, 26, 34))
dr = ImageDraw.Draw(sheet)
for k, j in enumerate(sel):
    t = d / f"{j['id']}.thumb.png"
    x, y = (k % cols) * tw, (k // cols) * (th + 28)
    if t.exists():
        im = Image.open(t).convert('RGB'); im.thumbnail((tw, th)); sheet.paste(im, (x, y))
    dr.text((x + 3, y + th + 2), f"{j['id']} {j['name'][:18]}", fill=(230, 230, 230))
    dr.text((x + 3, y + th + 14), f"f{j['faces']} {'x'.join(str(int(b-a)) for a,b in zip(j['bounds_min'],j['bounds_max']))}", fill=(160, 170, 180))
sheet.save(out); print(len(sel), out)
