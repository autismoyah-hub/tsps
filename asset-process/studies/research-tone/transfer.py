"""Nearest-face transfer between two meshes (Teacher-side experiments only: it reads the real mesh).

nearest(src, dst)   for every drawn face of dst: the drawn face of src nearest to it (three interior sample points
                    and the centroid vote; opposite-facing or other-translucency candidates are penalised).
"""
import sys, pathlib; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
import tl

def closest_dist(P, A, Bv, Cv):
    """Distance from points P [n,3] to triangles (A,B,C) [m,3] -> [n,m] (Ericson's closest point, vectorised)."""
    P = P[:, None, :]; A = A[None]; Bv = Bv[None]; Cv = Cv[None]
    ab, ac, ap = Bv - A, Cv - A, P - A
    d1 = (ab * ap).sum(-1); d2 = (ac * ap).sum(-1)
    bp = P - Bv; d3 = (ab * bp).sum(-1); d4 = (ac * bp).sum(-1)
    cp = P - Cv; d5 = (ab * cp).sum(-1); d6 = (ac * cp).sum(-1)
    vc = d1 * d4 - d3 * d2; vb = d5 * d2 - d1 * d6; va = d3 * d6 - d5 * d4
    den = va + vb + vc; den = np.where(np.abs(den) < 1e-12, 1e-12, den)
    v = vb / den; w = vc / den
    Q = A + ab * v[..., None] + ac * w[..., None]                                  # interior
    def setw(cond, pt):
        nonlocal Q
        Q = np.where(cond[..., None], pt, Q)
    e = lambda x: np.where(np.abs(x) < 1e-12, 1e-12, x)
    setw((va <= 0) & ((d4 - d3) >= 0) & ((d5 - d6) >= 0), Bv + (Cv - Bv) * ((d4 - d3) / e((d4 - d3) + (d5 - d6)))[..., None])
    setw((vb <= 0) & (d2 >= 0) & (d6 <= 0), A + ac * (d2 / e(d2 - d6))[..., None])
    setw((vc <= 0) & (d1 >= 0) & (d3 <= 0), A + ab * (d1 / e(d1 - d3))[..., None])
    setw((d6 >= 0) & (d5 <= d6), np.broadcast_to(Cv, Q.shape))
    setw((d3 >= 0) & (d4 <= d3), np.broadcast_to(Bv, Q.shape))
    setw((d1 <= 0) & (d2 <= 0), np.broadcast_to(A, Q.shape))
    return np.linalg.norm(P - Q, axis=-1)

def unit_normals(a):
    V, F = a['V'], a['F']; n = np.cross(V[F[:, 1]] - V[F[:, 0]], V[F[:, 2]] - V[F[:, 0]])
    return n / np.maximum(np.linalg.norm(n, axis=1), 1e-9)[:, None]

def nearest(src, dst, facing_pen=12.0, alpha_pen=30.0):
    """-> for each face of dst the index of a face of src (or -1 for undrawn dst faces), and the mean distance."""
    ds, dd = tl.drawn(src), tl.drawn(dst); si = np.nonzero(ds)[0]; di = np.nonzero(dd)[0]
    A, Bv, Cv = (src['V'][src['F'][si, k]] for k in range(3)); ns, nd = unit_normals(src)[si], unit_normals(dst)
    T = dst['V'][dst['F']]                                                      # [m,3,3]
    cen = T.mean(1); samples = [cen] + [cen * 0.4 + T[:, k] * 0.6 for k in range(3)]
    out = np.full(len(dst['F']), -1); dist = np.zeros(len(dst['F']))
    ta = src['A'][si] > 0
    for s0 in range(0, len(di), 200):
        idx = di[s0:s0 + 200]; score = np.zeros((len(idx), len(si))); dmin = np.zeros(len(idx))
        pen = np.where((nd[idx] @ ns.T) < 0, facing_pen, 0.0) + np.where((dst['A'][idx] > 0)[:, None] != ta[None], alpha_pen, 0.0)
        for S in samples:
            d = closest_dist(S[idx], A, Bv, Cv) + pen; score += 1.0 / (d + 1.0)
        dmin = closest_dist(cen[idx], A, Bv, Cv).min(1)
        out[idx] = si[score.argmax(1)]; dist[idx] = dmin
    return out, dist
