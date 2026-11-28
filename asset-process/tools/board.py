"""Labelled board of pictures. usage: board.py OUT COLS "label|path" ..."""
import sys
from PIL import Image, ImageDraw, ImageFont
out, cols = sys.argv[1], int(sys.argv[2])
items = [a.split('|', 1) for a in sys.argv[3:]]
ims = [Image.open(p).convert('RGB') for _, p in items]
w, h = ims[0].size
try: font = ImageFont.truetype('segoeui.ttf', 30)
except Exception: font = ImageFont.load_default()
pad = 56
rows = (len(ims) + cols - 1) // cols
board = Image.new('RGB', (cols * w, rows * (h + pad)), (16, 20, 26))
d = ImageDraw.Draw(board)
for k, ((label, _), im) in enumerate(zip(items, ims)):
    x, y = (k % cols) * w, (k // cols) * (h + pad)
    d.text((x + 14, y + 8), label, fill=(235, 235, 235), font=font)
    board.paste(im.resize((w, h)), (x, y + pad))
board.save(out); print(out, board.size)
