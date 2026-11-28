"""crop.py IN.png OUT.png X0 Y0 X1 Y1 [SCALE]: cut a region out of a picture, optionally enlarged (nearest)."""
import sys
from PIL import Image
im = Image.open(sys.argv[1]); x0, y0, x1, y1 = map(int, sys.argv[3:7])
im = im.crop((x0, y0, x1, y1))
if len(sys.argv) > 7:
    k = float(sys.argv[7]); im = im.resize((int(im.width * k), int(im.height * k)), Image.NEAREST)
im.save(sys.argv[2]); print(sys.argv[2], im.size)
