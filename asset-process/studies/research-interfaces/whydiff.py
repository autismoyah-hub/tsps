"""whydiff.py CMP_PREFIX STATE.jsonl ROOT(g.c): for each differing pixel between CMP_PREFIX-tsps.png and -ours.png,
name the topmost drawn component covering it, and summarise (component, kind, transparency, our value, theirs)."""
import sys, json, collections
from pathlib import Path
import numpy as np
from PIL import Image

sys.path.insert(0, str(Path(__file__).resolve().parent))
import iface

pre, st, root = sys.argv[1:4]
A = np.asarray(Image.open(pre + '-tsps.png').convert('RGB')).astype(int)
B = np.asarray(Image.open(pre + '-ours.png').convert('RGB')).astype(int)
order, lay, byid = iface.load_state(st)
g, c = (int(v) for v in root.split('.'))
rx, ry = lay[(g << 16) | c]['abs']
d = (A != B).any(-1)
ys, xs = np.nonzero(d)
cnt = collections.Counter(); ex = {}
for y, x in zip(ys, xs):
    X, Y = x + rx, y + ry
    top = None
    for i in order:
        p = lay[i]; w = byid[i]
        if p['hidden'] or w['kind'] in (0, 11):
            continue
        cl = p['clip']
        if cl[0] <= X < cl[2] and cl[1] <= Y < cl[3]:
            top = i
    w = byid.get(top, {})
    key = (top, w.get('kind'), w.get('transparency'), w.get('filled'), hex(w.get('colors', [0])[0]), w.get('sprites', [-1])[0], w.get('font'))
    cnt[key] += 1
    ex.setdefault(key, (tuple(B[y, x]), tuple(A[y, x])))
for k, n in cnt.most_common(12):
    print(n, 'comp', k[0] and hex(k[0]), 'kind', k[1], 'transp', k[2], 'filled', k[3], 'colour', k[4], 'sprite', k[5], 'font', k[6],
          'ours', ex[k][0], 'tsps', ex[k][1])
