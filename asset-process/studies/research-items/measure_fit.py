"""measure_fit.py: the mesh-pair fits used by measure_items.py (same topology test, rigid fit, per-axis fit)."""
import numpy as np


def same_topology(a, b):
    return a is not None and b is not None and len(a['V']) == len(b['V']) and len(a['F']) == len(b['F']) and np.array_equal(a['F'], b['F'])


def rigid_fit(A, B):
    """Best rotation R (det +1 or -1 allowed: mirror reported) and t with B ~ A R + t; returns rms, max, R, t, uniform scale."""
    A = A.astype(float); B = B.astype(float)
    ca, cb = A.mean(0), B.mean(0)
    H = (A - ca).T @ (B - cb)
    U, S, Vt = np.linalg.svd(H)
    R = U @ Vt
    sa = ((A - ca) ** 2).sum()
    scale = S.sum() / sa if sa > 0 else 1.0
    best = None
    for s in (1.0, scale):
        P = (A - ca) @ R * s + cb
        e = np.sqrt(((P - B) ** 2).sum(1))
        r = (float(np.sqrt((e ** 2).mean())), float(e.max()), R, cb - ca @ R * s, s)
        if best is None or r[0] < best[0] - 0.05: best = r
    return best


def axis_fit(A, B):
    """Per-axis scale and shift: B[:,k] ~ a_k A[:,k] + b_k. Returns rms, max, scales, shifts."""
    A = A.astype(float); B = B.astype(float)
    P = np.empty_like(B); sc = []; sh = []
    for k in range(3):
        va = A[:, k].var()
        a = ((A[:, k] - A[:, k].mean()) * (B[:, k] - B[:, k].mean())).mean() / va if va > 1e-9 else 1.0
        b = B[:, k].mean() - a * A[:, k].mean()
        P[:, k] = a * A[:, k] + b; sc.append(a); sh.append(b)
    e = np.sqrt(((P - B) ** 2).sum(1))
    return float(np.sqrt((e ** 2).mean())), float(e.max()), sc, sh


def euler_deg(R):
    """Describe a rotation matrix (row-vector convention) loosely: angle of rotation in degrees."""
    c = (np.trace(R) - 1) / 2
    return float(np.degrees(np.arccos(max(-1, min(1, c)))))


