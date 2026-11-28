"""Weights for a bone-weighted creature, by rule: numpy only (runs in Blender and in the venv).

A weight table is dense: W (vertices, bones), rows summing to 1. to_bytes() turns it into the file's form: per
vertex a list of (bone, byte) with the bytes summing to 255 (the client divides by 255 and does not normalise).
Positions are MODEL space (x, y down, z) throughout.
"""
import numpy as np


def shells(nv, F):
    """Connected component of each vertex (faces joined by shared vertices); unused vertices get their own."""
    parent = np.arange(nv)
    def find(x):
        while parent[x] != x: parent[x] = parent[parent[x]]; x = parent[x]
        return x
    for a, b, c in np.asarray(F):
        ra, rb, rc = find(a), find(b), find(c)
        if ra != rb: parent[rb] = ra
        ra = find(a)
        if rc != ra: parent[rc] = ra
    roots = np.array([find(i) for i in range(nv)]); _, lab = np.unique(roots, return_inverse=True)
    return lab


def weld_shells(V, F):
    """As shells, after joining vertices that sit on the same whole-unit position (parts that touch are one piece)."""
    V = np.rint(np.asarray(V)).astype(int); _, inv = np.unique(V, axis=0, return_inverse=True)
    lab = shells(int(inv.max()) + 1, inv[np.asarray(F)]); return lab[inv]


def segments(sk, stub=0.35):
    """[(bone, p0, p1)] in model space: a bone runs from its joint to each child joint; a leaf bone gets a short stub
    continuing its parent's direction (a joint of the game's skeleton has no length of its own)."""
    H = sk.head(); out = []
    for i in range(sk.n):
        kids = [c for c in sk.children[i] if np.linalg.norm(H[c] - H[i]) > 1e-6]
        if kids:
            for c in kids: out.append((i, H[i], H[c]))
        else:
            p = sk.parent[i]; d = H[i] - H[p] if p >= 0 else np.zeros(3)
            out.append((i, H[i], H[i] + d * stub))
    return out


def markers(sk):
    """Bones that are markers, not deforming bones: LEAVES resting exactly on the model origin (real bases carry them at
    the ends of limbs: Zebak has six). They take no weight and must not lend a segment to their parent."""
    H = sk.head(); return [i for i in range(sk.n) if not sk.children[i] and sk.parent[i] >= 0 and float(np.abs(H[i]).max()) < 0.5]


def segments_deform(sk, deform, stub=0.35, cut=(), shift=0.0):
    """As segments, for the bones with deform[i] true; a child that is a non-deforming leaf, or is in `cut`, lends no
    segment. shift: the share of each joint-to-joint stretch, at its CHILD end, that belongs to the child bone (on
    real creatures the skin just before a joint already rides the bone beyond it: RIG.md)."""
    H = sk.head(); out = []
    for i in range(sk.n):
        if not deform[i]: continue
        kids = [c for c in sk.children[i] if np.linalg.norm(H[c] - H[i]) > 1e-6 and (deform[c] or sk.children[c]) and c not in cut]
        if kids:
            for c in kids:
                mid = H[c] + (H[i] - H[c]) * shift; out.append((i, H[i], mid))
                if shift > 0 and deform[c]: out.append((c, mid, H[c]))
        else:
            p = sk.parent[i]; d = H[i] - H[p] if p >= 0 else np.zeros(3)
            out.append((i, H[i], H[i] + d * stub))
    return out


def point_segment(V, p0, p1):
    d = p1 - p0; L2 = float(d @ d)
    t = np.zeros(len(V)) if L2 < 1e-12 else np.clip(((V - p0) @ d) / L2, 0, 1)
    return np.linalg.norm(V - (p0 + t[:, None] * d), axis=1), t


def vertex_bone_distance(V, seg, nb):
    """(vertices, bones): distance from each vertex to the nearest segment of each bone (inf for a bone with none)."""
    D = np.full((len(V), nb), np.inf)
    for b, p0, p1 in seg:
        d, _ = point_segment(V, p0, p1); D[:, b] = np.minimum(D[:, b], d)
    return D


def adjacency(nv, F):
    import collections
    nb = collections.defaultdict(set)
    for a, b, c in np.asarray(F).tolist():
        nb[a] |= {b, c}; nb[b] |= {a, c}; nb[c] |= {a, b}
    return [sorted(nb[i]) for i in range(nv)]


def to_bytes(W, max_influences=4, floor=0.05, snap=0.0):
    """Dense rows -> the file's lists. Per vertex: keep the `max_influences` largest, drop shares under `floor`, give
    everything to the largest when it holds at least `snap` (0 = never), then bytes by largest remainder so the sum
    is exactly 255. Pairs are written heaviest first."""
    out = []
    for row in np.asarray(W, float):
        idx = np.argsort(-row)[:max_influences]; w = row[idx]; keep = w > 0
        idx, w = idx[keep], w[keep]
        if len(w) == 0: out.append([]); continue
        w = w / w.sum(); keep = w >= floor; keep[0] = True; idx, w = idx[keep], w[keep]; w = w / w.sum()
        if snap and w[0] >= snap: out.append([(int(idx[0]), 255)]); continue
        q = np.floor(w * 255).astype(int); rem = 255 - q.sum()
        for k in np.argsort(-(w * 255 - q))[:rem]: q[k] += 1
        out.append([(int(b), int(x)) for b, x in zip(idx, q) if x > 0])
    return out


def dense(weights, nb):
    W = np.zeros((len(weights), nb))
    for i, ws in enumerate(weights):
        for b, w in ws: W[i, b] += w / 255.0
    return W


def smooth(W, adj, rounds=1, keep=0.5):
    """Each vertex's row becomes keep x itself + (1 - keep) x the mean of its neighbours' rows."""
    W = W.copy()
    for _ in range(rounds):
        N = np.stack([W[a].mean(0) if a else W[i] for i, a in enumerate(adj)])
        W = keep * W + (1 - keep) * N
    return W
