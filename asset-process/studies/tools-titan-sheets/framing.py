from pathlib import Path
import numpy as np
from PIL import Image
root=Path(__file__).resolve().parent
for n in (12596,14147):
    sh=root/'sheets'/str(n)
    for directory,pat in [('actions','*.png'),('pose','*-lit.png')]:
        hit=[]
        for p in (sh/directory).glob(pat):
            im=np.asarray(Image.open(p).convert('RGB')); m=(im!=[48,43,37]).any(2)
            if m[0].any() or m[-1].any() or m[:,0].any() or m[:,-1].any():hit.append(p.name)
        print(n,directory,len(hit),hit[:20])

