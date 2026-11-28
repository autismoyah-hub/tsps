"""The bone tree of a creature as text (no weights are read): what an author needs to write a parts file.
usage: rig_tree.py WORKDIR [--depth N] [--colours]
Each line: bone, joint position (x, height, z), bones below it, the box their joints span. --colours: the mesh's colours
with face counts and the box of their vertices (x, height, z)."""
import sys, json, pathlib, collections
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
import riglib as R
B, RM = R.B, R.RM
a = sys.argv; work = pathlib.Path(a[1]).resolve(); man = json.loads((work / 'manifest.json').read_text()); src = work / man['src']
sk = RM.Skeleton(B.decode_base((src / man['base_file']).read_bytes())); H = sk.head().copy(); H[:, 1] *= -1
maxd = int(a[a.index('--depth') + 1]) if '--depth' in a else 99
def sub(i): return [i] + [y for c in sk.children[i] for y in sub(c)]
def show(i, ind):
    s = sub(i); lo, hi = H[s].min(0).round().astype(int), H[s].max(0).round().astype(int)
    print(f"{'  ' * ind}{i} at {H[i].round().astype(int).tolist()}: {len(s) - 1} below" + (f", box {lo.tolist()}..{hi.tolist()}" if len(s) > 1 else ''))
    if ind < maxd:
        for c in sk.children[i]: show(c, ind + 1)
show(sk.parent.index(-1), 0)
if '--colours' in a:
    m = B.decode_model((src / man['models'][0]).read_bytes()); V = np.array(m['vertices'], float); V[:, 1] *= -1; acc = collections.defaultdict(list)
    for f, c in zip(m['faces'], m['colors']): acc[c] += list(f)
    for c, idx in sorted(acc.items(), key=lambda kv: -len(kv[1])):
        P = V[sorted(set(idx))]; print(f"hsl_{(c >> 10) & 63}_{(c >> 7) & 7}_{c & 127}: {len(idx) // 3} faces, box {P.min(0).astype(int).tolist()}..{P.max(0).astype(int).tolist()}")
