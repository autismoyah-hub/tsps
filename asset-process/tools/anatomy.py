"""How a real model is built: rings, sharing, hard edges, colours. usage: anatomy.py MODEL_ID [MODEL_ID...]"""
import sys, math, collections, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import osrscache as o
from measure import area


def report(mid):
    m = o.model(mid)
    v, F = m['vertices'], m['faces']
    pos = collections.Counter(v)
    dup = sum(1 for p, n in pos.items() if n > 1)
    print(f'model {mid}: {len(v)} vertices at {len(pos)} positions ({dup} positions used by more than one vertex = hard seams), {len(F)} faces, format {m["format"]}')
    ex = [(min(p[i] for p in v), max(p[i] for p in v)) for i in range(3)]
    print('  bounds x,y,z', ex)
    # edges: shared by index (smooth) vs only by position (hard)
    e_idx = collections.Counter(); e_pos = collections.Counter()
    for f in F:
        for a, b in ((f[0], f[1]), (f[1], f[2]), (f[2], f[0])):
            e_idx[frozenset((a, b))] += 1
            e_pos[frozenset((v[a], v[b]))] += 1
    open_idx = sum(1 for n in e_idx.values() if n == 1)
    open_pos = sum(1 for n in e_pos.values() if n == 1)
    print(f'  edges: {len(e_idx)} by index, {open_idx} open by index, {open_pos} open by position -> {open_idx - open_pos} hard-seam edges, {open_pos} true borders (open shells)')
    # height levels
    ys = collections.Counter(p[1] for p in v)
    print('  height levels (y: vertices, max radius):')
    for y in sorted(ys):
        r = max(math.hypot(p[0], p[2]) for p in v if p[1] == y)
        print(f'    y={y:5d}: {ys[y]:3d} verts, r<={r:5.1f}')
    ar = collections.Counter(); n = collections.Counter()
    for f, c in zip(F, m['colors']):
        ar[c] += area(v, f); n[c] += 1
    tot = sum(ar.values())
    print('  colours (hue,sat,light: faces, share of area):')
    for c, a in ar.most_common():
        print(f'    {o.hsl_split(c)}: {n[c]:4d} faces {100*a/tot:5.1f}%')
    print('  priorities', dict(collections.Counter(m['priorities'])), 'alpha faces', sum(1 for a in m['alpha'] if a))
    return m


if __name__ == '__main__':
    for a in sys.argv[1:]:
        report(int(a))
