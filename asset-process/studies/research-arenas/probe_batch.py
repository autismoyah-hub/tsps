"""probe_batch.py OUT.png name:x:y:plane[:r] ...   plans of several map windows on one sheet (for locating arenas by eye)."""
import sys, pathlib
sys.dont_write_bytecode = True
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import arenalib as A, probe
from PIL import Image, ImageDraw
ims = []
for s in sys.argv[2:]:
    p = s.split(':'); name = p[0]; x, y, pl = map(int, p[1:4]); r = int(p[4]) if len(p) > 4 else 32
    W = A.World(x - r, y - r, x + r, y + r, pl)
    try: S = W.floor((x, y))
    except SystemExit: S = set()
    cell = max(3, 384 // (2 * r + 1))
    im = probe.plan(W, S, str(A.HERE / 'probe' / f'_b_{name}.png'), cell=cell, marks=[(x, y, 1, (0, 255, 255))], title=f'{name} ({x},{y},{pl}) walk {len(S)}')
    ims.append(im)
    print(name, 'walkable', len(S), 'bbox', (min(t[0] for t in S), min(t[1] for t in S), max(t[0] for t in S), max(t[1] for t in S)) if S else None)
w = max(i.size[0] for i in ims); h = max(i.size[1] for i in ims); cols = min(4, len(ims)); rows = (len(ims) + cols - 1) // cols
sheet = Image.new('RGB', (cols * w, rows * h))
for k, im in enumerate(ims): sheet.paste(im, ((k % cols) * w, (k // cols) * h))
sheet.save(sys.argv[1]); print(sys.argv[1], sheet.size)
