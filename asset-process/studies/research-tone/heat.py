"""Real | ours | absolute grey difference (heat) for chosen rows, both gate views. usage: heat.py NPC OURS.json OUT.png [rows]"""
import sys, pathlib; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
from PIL import Image
import tl
from tl import L
def run(npc, ours, out, rows=('bind',), zoom=3):
    seqs = L.sequences(npc); cam = tl.camera(npc); strips = []
    for yaw in (192, 1216):
        cells, _ = tl.draw([L.entry_real(npc, seqs), tl.entry(ours, seqs)], list(rows), cam, yaw)
        for row, (A, B) in zip(rows, cells):
            I = tl.mask(A) & tl.mask(B); d = np.abs(tl.grey(A) - tl.grey(B)); v = np.clip(d / 60, 0, 1)
            h = np.zeros(A.shape, np.uint8); h[:] = (20, 20, 24); h[I] = np.stack([40 + 215 * v, 40 + 150 * v * v, 40 + 0 * v], -1)[I].astype(np.uint8); h[tl.mask(A) ^ tl.mask(B)] = (0, 150, 0)
            ys, xs = np.nonzero(tl.mask(A) | tl.mask(B)); y0, y1, x0, x1 = max(ys.min() - 4, 0), ys.max() + 5, max(xs.min() - 4, 0), xs.max() + 5
            strips.append(np.concatenate([A[y0:y1, x0:x1], B[y0:y1, x0:x1], h[y0:y1, x0:x1]], 1)); print(row, yaw, 'tone', round(float(d[I].mean()), 3))
    w = max(s.shape[1] for s in strips); strips = [np.pad(s, ((0, 0), (0, w - s.shape[1]), (0, 0))) for s in strips]
    im = Image.fromarray(np.concatenate(strips, 0)); im = im.resize((im.width * zoom, im.height * zoom), Image.NEAREST); pathlib.Path(out).parent.mkdir(parents=True, exist_ok=True); im.save(out)
if __name__ == '__main__':
    run(int(sys.argv[1]), sys.argv[2], sys.argv[3], tuple(sys.argv[4].split(',')) if len(sys.argv) > 4 else ('bind',))
