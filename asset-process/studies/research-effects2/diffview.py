"""diffview.py OID [close]: where the by-id and file pictures of an object differ (render/research-effects2-faithful/diff-OID.png)."""
import sys, pathlib
sys.dont_write_bytecode = True; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import fx, ob
from fxlib import *
import numpy as np
from PIL import Image
oid = int(sys.argv[1]); close = len(sys.argv) > 2
p, d = ob.dump(oid, HERE / 'real' / f'loc-{oid}')
total = fx.life(f'loc:{oid}', 'loc'); cyc = fx.moments(10, total)
A, la = fx.render(f'loc:{oid}', 'loc', cyc, close=close, size=(480, 400))
B, lb = fx.render(f'locf:{p.as_posix()}', 'loc', cyc, close=close, size=(480, 400))
rows = []
for c, a, b in zip(cyc, A, B):
    m = (np.asarray(a) != np.asarray(b)).any(axis=2)
    ys, xs = np.nonzero(m)
    print(c, int(m.sum()), (xs.min(), ys.min(), xs.max(), ys.max()) if m.any() else '')
    dm = Image.fromarray((m * 255).astype('uint8')).convert('RGB')
    row = Image.new('RGB', (1440, 400)); row.paste(a, (0, 0)); row.paste(b, (480, 0)); row.paste(dm, (960, 0)); rows.append(row)
k = max(range(len(rows)), key=lambda i: (np.asarray(A[i]) != np.asarray(B[i])).any(axis=2).sum())
out = RENDER / 'research-effects2-faithful' / f'diff-{oid}.png'; rows[k].save(out); print(out, 'cycle', cyc[k])
print([l for l in la.splitlines() if 'loc add' in l])
