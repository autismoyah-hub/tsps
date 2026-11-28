"""Count differing pixels between pairs of pictures. usage: pixdiff.py A B [A2 B2 ...]"""
import sys
from PIL import Image, ImageChops
a = sys.argv[1:]
for p, q in zip(a[::2], a[1::2]):
    x, y = Image.open(p).convert('RGB'), Image.open(q).convert('RGB')
    if x.size != y.size:
        print(f'{p} {x.size} vs {q} {y.size}: SIZES DIFFER'); continue
    d = ImageChops.difference(x, y).convert('L').point(lambda v: 255 if v else 0)
    n = sum(1 for v in d.getdata() if v)
    print(f'{p} vs {q}: {n} of {x.width * x.height} pixels differ')
