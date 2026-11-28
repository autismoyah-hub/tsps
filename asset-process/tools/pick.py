"""Spread a name's distinct real objects across the id range and tile their three-quarter views.
usage: pick.py REGEX N OUT  (crops the THREE QUARTER panel from each sheet)"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, json, re, pathlib
from PIL import Image, ImageDraw
ARCH = pathlib.Path(_AP.fix(r'<REFERENCE_ARCHIVE>\objects\shape-10'))
rx, n, out = sys.argv[1], int(sys.argv[2]), sys.argv[3]
sel, seen = [], set()
for p in sorted(ARCH.glob('*.json'), key=lambda p: int(p.stem)):
    j = json.loads(p.read_text())
    k = tuple(j.get('model_ids') or [])
    if not k or k in seen or not re.fullmatch(rx, j.get('name') or '', re.I) or (j.get('faces') or 0) < 60: continue
    seen.add(k); sel.append(j)
step = max(1, len(sel) // n)
sel = sel[::step][:n]
cols = 6; cw = 352; ch = 340
sheet = Image.new('RGB', (cols * cw, ((len(sel) + cols - 1) // cols) * (ch + 30)), (20, 26, 34)); d = ImageDraw.Draw(sheet)
for k, j in enumerate(sel):
    im = Image.open(ARCH / f"{j['id']}.png").convert('RGB')
    crop = im.crop((352, 572, 704, 912))
    x, y = (k % cols) * cw, (k // cols) * (ch + 30)
    sheet.paste(crop, (x, y))
    d.text((x + 4, y + ch + 4), f"{j['id']} f{j['faces']} " + 'x'.join(str(int(b - a)) for a, b in zip(j['bounds_min'], j['bounds_max'])), fill=(230, 230, 230))
sheet.save(out); print(len(sel), 'of', len(seen), out)
