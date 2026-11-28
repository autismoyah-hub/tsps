"""Where the surviving measure sits: faces on concave in-paint folds of 30 degrees or more tinted red (C9 `concave`),
ours beside its nearest real one, close, yaw 20 and 160 (front and back three-quarter). -> boards/hot-*.png"""
import sys, pathlib
import numpy as np, cv2
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import gaplib as g, measures as M
from board import pad_to, label, rgb2bgr
cl = g.cl

def tinted(c, yaw):
    M.prep(c); E = c['E']
    same = c['C'][c['e_a'][E]] == c['C'][c['e_b'][E]]
    k = E[same & (c['e_dih'][E] <= -30)]
    tint = {int(f): (230, 30, 30) for f in np.concatenate([c['e_a'][k], c['e_b'][k]])}
    return cl.view(c, yaw=yaw, pitch=10, size=420, tint=tint)[0]

for ours in (sys.argv[1:] or g.BEST):
    real = g.NEAREST[ours]
    co, cr = g.load(g.ALL[ours]), g.load(g.ALL[real])
    row = [pad_to(tinted(cr, y), 420, 420) for y in (20, 160)] + [np.zeros((420, 6, 3), np.uint8)] + [pad_to(tinted(co, y), 420, 420) for y in (20, 160)]
    img = label(np.hstack(row), f'concave in-paint folds >= 30 deg in red: left {real} (real), right {ours} (ours)')
    out = HERE / 'boards' / f"hot-{ours.replace(' ', '-').lower()}.png"
    cv2.imwrite(str(out), rgb2bgr(img)); print(out)
