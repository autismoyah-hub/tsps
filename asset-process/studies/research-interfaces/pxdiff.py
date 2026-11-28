"""pxdiff.py A.png B.png [DIFF.png]: count differing pixels (RGB exact) and their bounding box."""
import sys
import numpy as np
from PIL import Image


def diff(a, b):
    A = np.asarray(Image.open(a).convert('RGB')).astype(int)
    B = np.asarray(Image.open(b).convert('RGB')).astype(int)
    if A.shape != B.shape:
        return None, A.shape, B.shape
    d = (A != B).any(-1)
    return d, A, B


if __name__ == '__main__':
    d, A, B = diff(sys.argv[1], sys.argv[2])
    if d is None:
        print('size mismatch', A, B); sys.exit(2)
    n = int(d.sum())
    box = None
    if n:
        ys, xs = np.nonzero(d)
        box = (int(xs.min()), int(ys.min()), int(xs.max()), int(ys.max()))
    print(f'{n} of {d.size} pixels differ; box {box}; max channel delta {int(np.abs(A - B).max())}')
    if len(sys.argv) > 3:
        out = (A * 0.35).astype(np.uint8)
        out[d] = [255, 0, 255]
        Image.fromarray(out).save(sys.argv[3])
