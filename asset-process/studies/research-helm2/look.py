"""look.py: sheets for LOOKING. usage: look.py [keys...]  -> work/look-game.png, work/look-head.png"""
import sys
import numpy as np
from PIL import Image, ImageDraw
import pathlib; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent)); import h2

keys = sys.argv[1:] or h2.ORDER
es = [h2.entry(k) for k in keys]
rows = []
for yaw in (0, 1024):
    a = h2.draw(f'look-game-{yaw}', es, yaw=yaw, zoom=300, cell=(150, 300))
    rows.append(a)
g = np.concatenate(rows, 0).astype(np.uint8)
im = Image.fromarray(g); d = ImageDraw.Draw(im)
for i, k in enumerate(keys): d.text((i * 150 + 4, 2), k, fill=(255, 255, 255))
im.save(h2.WORK / 'look-game.png')
rows = []
for yaw in (0, 256, 512, 1024):
    a = h2.draw(f'look-head-{yaw}', es, yaw=yaw, zoom=130, cell=(200, 220), aim='head')
    rows.append(a)
g = np.concatenate(rows, 0).astype(np.uint8)
im = Image.fromarray(g); d = ImageDraw.Draw(im)
for i, k in enumerate(keys): d.text((i * 200 + 4, 2), k, fill=(255, 255, 255))
im.save(h2.WORK / 'look-head.png')
print(im.size)
