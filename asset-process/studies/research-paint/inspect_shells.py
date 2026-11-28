"""Shells of a creature with their place, size and paint. usage: inspect_shells.py ENTRY"""
import sys, pathlib, collections
import numpy as np
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import paintlib as pl
def shells(sk):
    F = sk['F']; par = {}
    def find(i):
        while par.setdefault(i, i) != i: par[i] = par[par[i]]; i = par[i]
        return i
    for f in F:
        a = find(int(f[0]))
        for v in f[1:]: par[find(int(v))] = a
    root = np.array([find(int(f[0])) for f in F]); u, inv = np.unique(root, return_inverse=True)
    return inv, len(u)
if __name__ == '__main__':
    sk = pl.skin(sys.argv[1]); sh, n = shells(sk); mats, mat_of = pl.materials(sk)
    V = sk['V']; print('height axis: y down; ground y max =', V[np.unique(sk['F'])][:, 1].max())
    order = sorted(range(n), key=lambda j: -(sh == j).sum())
    for j in order:
        s = sh == j; P = V[np.unique(sk['F'][s])]; lo, hi = P.min(0), P.max(0)
        col = collections.Counter()
        for c, a in zip(sk['C'][s], sk['area'][s]): col[int(c)] += a
        labs = collections.Counter(sk['L'][sk['F'][s]].ravel().tolist())
        print(f"shell {j:2d} faces {s.sum():4d} area {100 * sk['area'][s].sum() / sk['tot']:5.1f}%  x {lo[0]:5.0f}..{hi[0]:5.0f} h {-hi[1]:5.0f}..{-lo[1]:5.0f} z {lo[2]:5.0f}..{hi[2]:5.0f}  labels {len(labs)} "
              + ' '.join('%d.%d.%d:%.0f%%' % (*pl.hsl(c), 100 * a / sk['area'][s].sum()) for c, a in col.most_common(6)))
