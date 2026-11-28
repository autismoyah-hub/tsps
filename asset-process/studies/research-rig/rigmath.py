"""Shared mathematics of the Blender route for skeletal (bone-weighted) creatures. numpy only: runs inside Blender's
Python and in the project's venv alike.

SPACES
  model space   what the model file stores: x right, y DOWN, z (the head end of a creature is -z)
  bone space    what the client's skin step works in: (x, -y, -z) of model space. Rest matrices and keyframe channels
                live here (SKELETAL.md section 5).
  Blender       the project's convention (tools/blender_kit.py): 1 Blender unit = 128 game units, +Z up,
                Blender (X, Y, Z) = (model x, model z, -model y) / 128 = (bone x, -bone z, bone y) / 128.
                Bone space -> Blender is a pure rotation and a scale (no mirror), so rotations carry over unchanged in
                sense: CS below.
THE CLIENT'S POSE (skeletal.rs, ported here in float64)
  local(bone) = compose(rot[3], tran[3], scale[3]): T x diag(scale) x R, R = the quaternion turned about z, then x,
  then y; global = parent global x local; skinning matrix K = global x inverse(global at rest).
BLENDER'S POSE  P(bone) = P(parent) x (A(parent)^-1 x A(bone)) x basis(bone)   (inherit_scale FULL), root: A x basis;
  deform matrix = P x A^-1, A = bone.matrix_local (armature space rest). So the two agree when
        P = CS x K x CS^-1 x A              (any A: the Blender bone's own orientation is FREE)
  to_blender_pose() and from_blender_pose() are that sentence both ways.
"""
import math, struct
import numpy as np

FMAX = struct.unpack('>f', b'\x7f\x7f\xff\xff')[0]
UNIT = 128.0
C3 = np.array([[1.0, 0, 0], [0, 0, -1.0], [0, 1.0, 0]])          # Blender = C3 @ bone space (then / 128)
CS = np.eye(4); CS[:3, :3] = C3 / UNIT
CSI = np.linalg.inv(CS)
MODEL_TO_BLENDER = np.array([[1.0, 0, 0], [0, 0, 1.0], [0, -1.0, 0]]) / UNIT      # Blender = this @ model (x, y down, z)


def f32(v): return struct.unpack('>f', struct.pack('>f', v))[0]
def col(m16): return np.array(m16, float).reshape(4, 4).T                 # the file's column-major 16 -> matrix
def flat(M): return [float(x) for x in np.asarray(M).T.reshape(16)]


# ------------------------------------------------------------------------------------------ the client's compose
def compose(rot, tran, scale):
    """skeletal.rs compose, vectorised: rot, tran, scale (..., 3) -> (..., 4, 4)."""
    rot = np.asarray(rot, float); tran = np.asarray(tran, float); scale = np.asarray(scale, float)
    q = np.zeros(rot.shape[:-1] + (4,)); q[..., 3] = 1.0
    for axis in (2, 0, 1):
        half = rot[..., axis] * 0.5; r = np.zeros_like(q); r[..., axis] = np.sin(half); r[..., 3] = np.cos(half)
        a, b = q, r
        q = np.stack([b[..., 1] * a[..., 2] + (a[..., 3] * b[..., 0] + a[..., 0] * b[..., 3]) - a[..., 1] * b[..., 2],
                      a[..., 0] * b[..., 2] + (a[..., 1] * b[..., 3] - b[..., 0] * a[..., 2] + b[..., 1] * a[..., 3]),
                      a[..., 3] * b[..., 2] + (a[..., 1] * b[..., 0] + a[..., 2] * b[..., 3] - b[..., 1] * a[..., 0]),
                      b[..., 3] * a[..., 3] - a[..., 0] * b[..., 0] - b[..., 1] * a[..., 1] - a[..., 2] * b[..., 2]], -1)
    x, y, z, w = q[..., 0], q[..., 1], q[..., 2], q[..., 3]
    M = np.zeros(rot.shape[:-1] + (4, 4)); M[..., 3, 3] = 1.0
    M[..., 0, 0] = w * w + x * x - z * z - y * y; M[..., 1, 0] = 2 * (x * y + w * z); M[..., 2, 0] = 2 * (x * z - w * y)
    M[..., 0, 1] = 2 * (x * y - w * z); M[..., 1, 1] = y * y + w * w - x * x - z * z; M[..., 2, 1] = 2 * (y * z + w * x)
    M[..., 0, 2] = 2 * (w * y + x * z); M[..., 1, 2] = 2 * (y * z - w * x); M[..., 2, 2] = z * z + w * w - y * y - x * x
    M[..., :3, :3] *= scale[..., :, None]                                  # the client scales ROWS: diag(scale) x R
    M[..., :3, 3] = tran
    return M


def decompose(L):
    """The inverse of compose: local matrices (..., 4, 4) -> rot, tran, scale (..., 3) with L = T x diag(scale) x R.
    The angles are the client's own rest-angle formula (skeletal.rs `rotations` of the inverse), so a matrix that
    compose made comes back as the same three numbers (away from the pole |rot.x| = 90 degrees, where the client
    itself folds z into y). `residual` = how far L is from that form (non-zero when rows are not orthogonal)."""
    L = np.asarray(L, float); A = L[..., :3, :3]
    scale = np.linalg.norm(A, axis=-1)
    det = np.linalg.det(A)
    R = A / np.where(scale == 0, 1, scale)[..., :, None]
    x = -np.arcsin(np.clip(R[..., 1, 2], -1, 1))
    pole = np.abs(np.cos(x)) <= 0.005
    y = np.where(pole, np.where(R[..., 1, 2] < 0, 1, -1) * np.arctan2(R[..., 0, 1], R[..., 0, 0]), np.arctan2(R[..., 0, 2], R[..., 2, 2]))
    z = np.where(pole, 0.0, np.arctan2(R[..., 1, 0], R[..., 1, 1]))
    rot = np.stack([x, y, z], -1)
    back = compose(rot, L[..., :3, 3], scale)
    residual = np.abs(back[..., :3, :3] - A).max(axis=(-1, -2))
    return rot, L[..., :3, 3].copy(), scale, residual, det


# ------------------------------------------------------------------------------------------------ the skeleton
class Skeleton:
    """A decoded frame base's bones: parents, local and global rest matrices (bone space), evaluation order."""
    def __init__(self, base):
        self.n = len(base['bones']); self.parent = [b['parent'] for b in base['bones']]
        self.local = np.array([col(b['matrices'][0]) for b in base['bones']]) if self.n else np.zeros((0, 4, 4))
        self.order = []; seen = [False] * self.n
        def visit(i):
            if seen[i]: return
            if self.parent[i] >= 0: visit(self.parent[i])
            seen[i] = True; self.order.append(i)
        import sys; sys.setrecursionlimit(max(10000, sys.getrecursionlimit()))
        for i in range(self.n): visit(i)
        self.rest = self.globals(self.local)
        self.rest_inv = np.linalg.inv(self.rest)
        r, t, s, _, _ = decompose(self.local); self.rest_channels = np.concatenate([r, t, s], -1)       # (n, 9): rot, tran, scale
        # the client takes rest SCALE from column lengths (Rig::new); identical when the rest has no scale
        self.children = [[] for _ in range(self.n)]
        for i, p in enumerate(self.parent):
            if p >= 0: self.children[p].append(i)
        self.depth = [0] * self.n
        for i in self.order: self.depth[i] = self.depth[self.parent[i]] + 1 if self.parent[i] >= 0 else 0
    def globals(self, local):
        """local (..., n, 4, 4) -> global, parents resolved in order."""
        G = np.empty_like(local)
        for i in self.order:
            p = self.parent[i]
            G[..., i, :, :] = local[..., i, :, :] if p < 0 else G[..., p, :, :] @ local[..., i, :, :]
        return G
    def locals_from_globals(self, G):
        Lc = np.empty_like(G)
        for i in range(self.n):
            p = self.parent[i]
            if p < 0: Lc[..., i, :, :] = G[..., i, :, :]
            else:
                try: Lc[..., i, :, :] = np.linalg.inv(G[..., p, :, :]) @ G[..., i, :, :]
                except np.linalg.LinAlgError: Lc[..., i, :, :] = np.linalg.pinv(G[..., p, :, :]) @ G[..., i, :, :]      # a parent scaled to nothing
        return Lc
    def skin_matrices(self, channels):
        """channels (..., n, 9) -> K (..., n, 4, 4) in bone space."""
        Lc = compose(channels[..., 0:3], channels[..., 3:6], channels[..., 6:9])
        return self.globals(Lc) @ self.rest_inv
    def head(self):
        """Joint positions at rest in MODEL space (x, y down, z)."""
        p = self.rest[:, :3, 3]; return np.stack([p[:, 0], -p[:, 1], -p[:, 2]], -1)


def skin(V_model, weights, K):
    """The client's Model::skin in float64. V_model (nv, 3) as stored; weights [[(bone, w)]]; K (nb, 4, 4) -> (nv, 3)."""
    V = np.asarray(V_model, float); P = np.concatenate([V[:, :1], -V[:, 1:2], -V[:, 2:3], np.ones((len(V), 1))], 1)
    out = V.copy(); nb = len(K)
    for i, ws in enumerate(weights):
        if not ws: continue
        M = np.zeros((4, 4))
        for b, w in ws:
            if b < nb: M += K[b] * (w / 255.0)
        q = M @ P[i]; out[i] = (q[0], -q[1], -q[2])
    return out


def weight_matrix(weights, nb):
    W = np.zeros((len(weights), nb))
    for i, ws in enumerate(weights):
        for b, w in ws:
            if b < nb: W[i, b] += w / 255.0
    return W


def skin_fast(V_model, W, K):
    """As skin, with a dense weight matrix W (nv, nb) and K (..., nb, 4, 4) -> (..., nv, 3)."""
    V = np.asarray(V_model, float); P = np.concatenate([V[:, :1], -V[:, 1:2], -V[:, 2:3], np.ones((len(V), 1))], 1)
    M = np.einsum('vb,...bij->...vij', W, K)
    q = np.einsum('...vij,vj->...vi', M, P)
    none = W.sum(1) == 0
    out = np.stack([q[..., 0], -q[..., 1], -q[..., 2]], -1)
    out[..., none, :] = V[none]
    return out


# ---------------------------------------------------------------------------------- Blender pose <-> game channels
def to_blender_pose(K, A):
    """K (..., n, 4, 4) bone-space skinning matrices, A (n, 4, 4) Blender rest (matrix_local) -> P armature-space pose."""
    return CS @ K @ CSI @ A
def from_blender_pose(P, A_inv):
    return CSI @ P @ A_inv @ CS
def basis_from_pose(P, A, parent, order):
    """Blender's pose-bone basis matrices (what location / rotation / scale channels hold) from armature-space P."""
    B = np.empty_like(P); Ai = np.linalg.inv(A); Pi = np.linalg.inv(P)
    for i in order:
        p = parent[i]
        B[..., i, :, :] = Ai[i] @ P[..., i, :, :] if p < 0 else np.linalg.inv(Ai[p] @ A[i]) @ Pi[..., p, :, :] @ P[..., i, :, :]
    return B
def basis_from_local(Lc, A, rest, parent):
    """The same basis matrices straight from the client's LOCAL matrices Lc (..., n, 4, 4) and the rest globals: no
    posed matrix is inverted, so a bone scaled to nothing (a corpse's hidden parts) is no obstacle.
    basis = A^-1 x Q(parent) x (CS x local x CS^-1) x Q^-1 x A, Q = CS x rest global x CS^-1."""
    Q = CS @ rest @ CSI; Qi = np.linalg.inv(Q); Ai = np.linalg.inv(A); B = np.empty_like(Lc)
    for i in range(len(parent)):
        p = parent[i]; M = CS @ Lc[..., i, :, :] @ CSI @ Qi[i] @ A[i]
        B[..., i, :, :] = Ai[i] @ (M if p < 0 else Q[p] @ M)
    return B
def pose_from_basis(B, A, parent, order):
    P = np.empty_like(B); Ai = np.linalg.inv(A)
    for i in order:
        p = parent[i]
        P[..., i, :, :] = A[i] @ B[..., i, :, :] if p < 0 else P[..., p, :, :] @ (Ai[p] @ A[i]) @ B[..., i, :, :]
    return P
def trs_from_matrix(B):
    """Blender's own decomposition: loc, quaternion (w, x, y, z), scale, with basis ~ T x R x diag(scale)."""
    loc = B[..., :3, 3].copy(); M = B[..., :3, :3]
    sc = np.linalg.norm(M, axis=-2)
    sc = np.where(np.linalg.det(M)[..., None] < 0, -sc, sc)
    R = M / np.where(sc == 0, 1, sc)[..., None, :]
    return loc, quat_from_matrix(R), sc
def matrix_from_trs(loc, quat, sc):
    w, x, y, z = (quat[..., k] for k in range(4))
    n = np.sqrt(w * w + x * x + y * y + z * z); w, x, y, z = w / n, x / n, y / n, z / n
    M = np.zeros(loc.shape[:-1] + (4, 4)); M[..., 3, 3] = 1
    M[..., 0, 0] = 1 - 2 * (y * y + z * z); M[..., 0, 1] = 2 * (x * y - w * z); M[..., 0, 2] = 2 * (x * z + w * y)
    M[..., 1, 0] = 2 * (x * y + w * z); M[..., 1, 1] = 1 - 2 * (x * x + z * z); M[..., 1, 2] = 2 * (y * z - w * x)
    M[..., 2, 0] = 2 * (x * z - w * y); M[..., 2, 1] = 2 * (y * z + w * x); M[..., 2, 2] = 1 - 2 * (x * x + y * y)
    M[..., :3, :3] *= sc[..., None, :]
    M[..., :3, 3] = loc
    return M
def quat_from_matrix(R):
    """Rotation matrices (..., 3, 3), orthonormalised by SVD first -> quaternions (w, x, y, z)."""
    U, _, Vt = np.linalg.svd(R); R = U @ Vt
    R = np.where((np.linalg.det(R) < 0)[..., None, None], U @ np.diag([1, 1, -1.0]) @ Vt, R)
    m00, m11, m22 = R[..., 0, 0], R[..., 1, 1], R[..., 2, 2]
    q = np.empty(R.shape[:-2] + (4,))
    tr = m00 + m11 + m22
    cand = np.stack([tr, m00 - m11 - m22, m11 - m00 - m22, m22 - m00 - m11], -1); k = cand.argmax(-1)
    s = np.sqrt(np.maximum(1 + np.take_along_axis(cand, k[..., None], -1)[..., 0], 1e-30)) * 2
    w0 = np.stack([0.25 * s, (R[..., 2, 1] - R[..., 1, 2]) / s, (R[..., 0, 2] - R[..., 2, 0]) / s, (R[..., 1, 0] - R[..., 0, 1]) / s], -1)
    w1 = np.stack([(R[..., 2, 1] - R[..., 1, 2]) / s, 0.25 * s, (R[..., 0, 1] + R[..., 1, 0]) / s, (R[..., 0, 2] + R[..., 2, 0]) / s], -1)
    w2 = np.stack([(R[..., 0, 2] - R[..., 2, 0]) / s, (R[..., 0, 1] + R[..., 1, 0]) / s, 0.25 * s, (R[..., 1, 2] + R[..., 2, 1]) / s], -1)
    w3 = np.stack([(R[..., 1, 0] - R[..., 0, 1]) / s, (R[..., 0, 2] + R[..., 2, 0]) / s, (R[..., 1, 2] + R[..., 2, 1]) / s, 0.25 * s], -1)
    q = np.where((k == 0)[..., None], w0, np.where((k == 1)[..., None], w1, np.where((k == 2)[..., None], w2, w3)))
    return q
def continuous_quats(q):
    """Flip signs along axis 0 (time) so neighbours never sit on opposite hemispheres (Blender interpolates channels)."""
    q = q.copy()
    for t in range(1, len(q)):
        flip = (q[t] * q[t - 1]).sum(-1) < 0; q[t][flip] *= -1
    return q


# ------------------------------------------------------------------------------------------- the client's curves
def evaluate(curve, time):
    """curve.rs Curve::evaluate in float64 for one time. Hold, step-to-next and unweighted (Hermite) segments are the
    client's arithmetic; a WEIGHTED segment is solved by bisection (the client clamps its handles first: not ported,
    exact only for handles that need no clamping)."""
    keys = curve['keys']; first, last = keys[0], keys[-1]
    if time < first[0]: return first[1] if curve['before'] == 0 else _extra(curve, time)
    if time > last[0]: return last[1] if curve['after'] == 0 else _extra(curve, time)
    ti = int(time); lo, hi = 0, len(keys)
    while lo < hi:
        mid = (lo + hi) // 2
        if keys[mid][0] <= ti: lo = mid + 1
        else: hi = mid
    i = max(lo - 1, 0); a = keys[i]
    if a[4] == 0.0 and a[5] == 0.0: return a[1]
    if a[4] == FMAX and a[5] == FMAX:
        return keys[i + 1][1] if (time != a[0] and i + 1 < len(keys)) else a[1]
    if i + 1 >= len(keys): return a[1]
    b = keys[i + 1]; x0, y0, x3, y3 = float(a[0]), a[1], float(b[0]), b[1]
    third = 0.33333334
    x1 = x0 + third * a[4]; y1 = a[5] * third + y0; x2 = x3 - third * b[2]; y2 = y3 - b[3] * third; span = x3 - x0
    if not curve['weighted']:
        s0 = (y1 - y0) / (x1 - x0) if x1 - x0 != 0 else 0.0; s1 = (y3 - y2) / (x3 - x2) if x3 - x2 != 0 else 0.0
        dy = y3 - y0; inv = 1.0 / (span * span); m0 = span * s0; m1 = span * s1
        c0 = inv * (m0 + m1 - dy - dy) / span; c1 = (dy + dy + dy - m0 - m0 - m1) * inv; dt = time - x0
        return y0 + dt * (s0 + (c1 + dt * c0) * dt)
    u = min(max((x1 - x0) / span, 0.0), 1.0); v = min(max((x2 - x0) / span, 0.0), 1.0); frac = (time - x0) / span
    bez = lambda t, p1, p2: 3 * (1 - t) ** 2 * t * p1 + 3 * (1 - t) * t * t * p2 + t ** 3
    lo, hi = 0.0, 1.0
    for _ in range(60):
        mid = (lo + hi) / 2
        if bez(mid, u, v) < frac: lo = mid
        else: hi = mid
    t = (lo + hi) / 2
    return (1 - t) ** 3 * y0 + 3 * (1 - t) ** 2 * t * y1 + 3 * (1 - t) * t * t * y2 + t ** 3 * y3
def _extra(curve, time): raise NotImplementedError('curve extrapolation modes 1 to 4: no real file uses them')


def channel_table(anim, skel, times):
    """Every bone channel of a decoded keyframe file evaluated at each time: (len(times), n bones, 9); channels the
    file does not supply keep the rest value, as Rig does. Later channels win, as in the client."""
    out = np.repeat(skel.rest_channels[None], len(times), 0)
    for c in anim['channels']:
        if c['kind'] != 1 or not (1 <= c['channel'] <= 9) or not (0 <= c['target'] < skel.n): continue
        cv = c['curve']
        out[:, c['target'], c['channel'] - 1] = [evaluate(cv, float(t)) for t in times]
    return out


def step_times(table):
    """The fewest key times from which 'each key steps to the next' (what real files do) reproduces every frame of a
    per-cycle table (n + 1 rows, frames 0..n): keep f when the pose at f differs from the pose at f + 1, and n."""
    n = len(table) - 1
    return [f for f in range(n) if f == 0 or not np.array_equal(table[f], table[f + 1])] + [n]


# ------------------------------------------------------------------------------------- writing a keyframe file
BYTE0 = {1: 0, 2: 0, 3: 0, 4: 1, 5: 1, 6: 1, 7: 3, 8: 3, 9: 3}
def baked_curve(times, values, rest, n, far=None):
    """One channel as real files key it: a key at -1 holding the rest value, a key at each time whose out-tangent
    (FLT_MAX, FLT_MAX) makes the client jump to the NEXT key's value, a closing key far beyond the range.
    A channel that never changes is written in the real files' four-key form."""
    far = far or max(4 * n, n + 100); v = [f32(x) for x in values]
    keys = [[-1, f32(rest), 1.0, 0.0, 1.0, 0.0]]
    if all(x == v[0] for x in v):
        keys += [[0, v[0], 1.0, 0.0, FMAX, FMAX], [n, v[0], 1.0, 0.0, 1.0, 0.0], [far, v[0], 1.0, 0.0, 1.0, 0.0]]
    else:
        prev = None
        for t, x in zip(times, v):
            keys.append([int(t), x, float(t - prev) if prev is not None else 1.0, 0.0, FMAX, FMAX]); prev = t
        keys.append([far, v[-1], 1.0, 0.0, 1.0, 0.0])
    return keys
def spline_curve(times, values, slopes, rest, n, far=None):
    """One channel as a TRUE curve: sparse keys with tangents (1, slope); the client interpolates every cycle as a
    cubic Hermite (curve.rs, unweighted)."""
    far = far or max(4 * n, n + 100); v = [f32(x) for x in values]
    keys = [[-1, f32(rest), 1.0, 0.0, 0.0, 0.0]]              # hold the rest value until time 0
    if all(x == v[0] for x in v):
        return keys + [[0, v[0], 1.0, 0.0, FMAX, FMAX], [n, v[0], 1.0, 0.0, 1.0, 0.0], [far, v[0], 1.0, 0.0, 1.0, 0.0]]
    for t, x, s in zip(times, v, slopes): keys.append([int(t), x, 1.0, f32(s), 1.0, f32(s)])
    keys[-1][4:6] = [0.0, 0.0]                                # after the last key: hold
    keys.append([far, v[-1], 1.0, 0.0, 1.0, 0.0])
    return keys
def unwrap(a):
    """Angles along axis 0 made continuous (a decomposition returns each one in -pi..pi)."""
    a = a.copy()
    for t in range(1, len(a)):
        d = a[t] - a[t - 1]; a[t] -= 2 * math.pi * np.round(d / (2 * math.pi))
    return a


def self_test():
    rng = np.random.default_rng(239)
    rot = rng.uniform(-1.5, 1.5, (2000, 3)); rot[:, 1:] = rng.uniform(-3.1, 3.1, (2000, 2)); tr = rng.uniform(-200, 200, (2000, 3)); sc = rng.uniform(0.5, 2, (2000, 3))
    M = compose(rot, tr, sc); r2, t2, s2, res, det = decompose(M)
    print('compose -> decompose: angle error', float(np.abs(r2 - rot).max()), 'scale', float(np.abs(s2 - sc).max()), 'residual', float(res.max()))
    # quaternion round trip
    R = compose(rot, np.zeros_like(tr), np.ones_like(sc))[..., :3, :3]; q = quat_from_matrix(R)
    R2 = matrix_from_trs(np.zeros((2000, 3)), q, np.ones((2000, 3)))[..., :3, :3]
    print('matrix -> quaternion -> matrix:', float(np.abs(R - R2).max()), ' det C3', float(np.linalg.det(C3)))
if __name__ == '__main__': self_test()
