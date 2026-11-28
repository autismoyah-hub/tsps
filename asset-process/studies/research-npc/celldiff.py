"""Enlarge two cells of a harness sheet and mark where they differ.

usage: celldiff.py SHEET.png COLS ROWS A:B ROW OUT.png [--scale N] [--crop X0,Y0,X1,Y1]
SHEET has COLS columns and ROWS rows of equal cells; A and B are 1-based columns, ROW a 1-based row.
Writes A | B | A with differing pixels painted magenta, each enlarged (default 2x), and prints the count and the
bounding box of the differing pixels in cell coordinates. --crop cuts the cell first (cell pixels).
"""
import sys
from PIL import Image
a = sys.argv
sheet = Image.open(a[1]).convert('RGB'); cols, rows = int(a[2]), int(a[3])
A, B = (int(v) - 1 for v in a[4].split(':')); row = int(a[5]) - 1
scale = int(a[a.index('--scale') + 1]) if '--scale' in a else 2
cw, ch = sheet.width // cols, sheet.height // rows
ca = sheet.crop((A * cw, row * ch, A * cw + cw, row * ch + ch)); cb = sheet.crop((B * cw, row * ch, B * cw + cw, row * ch + ch))
if '--crop' in a:
    box = tuple(int(v) for v in a[a.index('--crop') + 1].split(','))
    ca, cb = ca.crop(box), cb.crop(box)
m = ca.copy(); pa, pb, pm = ca.load(), cb.load(), m.load()
n = 0; xs = []; ys = []
for y in range(ca.height):
    for x in range(ca.width):
        if pa[x, y] != pb[x, y]:
            pm[x, y] = (255, 0, 255); n += 1; xs.append(x); ys.append(y)
out = Image.new('RGB', (ca.width * 3 * scale, ca.height * scale))
for k, im in enumerate((ca, cb, m)):
    out.paste(im.resize((ca.width * scale, ca.height * scale), Image.NEAREST), (k * ca.width * scale, 0))
out.save(a[6])
print(a[6], out.size, 'differing pixels', n, 'box', (min(xs), min(ys), max(xs), max(ys)) if xs else None)
