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

# ------------------------------------------------------------------------------------------- joint rings by rule
# Measured on 49 real weighted creatures, 3,072 bone-to-child boundaries (ring_census.py, ring_profile.py,
# ring_fit.py, ring_fit2.py, ring_table.py; RIG2.md section 1). What the numbers say:
#   - WHETHER a joint is blended is the creature's STYLE far more than anything about the joint: creatures with under
#     40% one-bone vertices blend 61 to 80% of their joints, creatures over 65% blend 15 to 32%. Kind of joint, bend
#     range and the boundary's shape each move it by 0.2 at most.
#   - WHERE: the ring of vertices touching the other bone's region (hop 1), within about half a limb radius of the
#     joint plane, carries the bone across; on soft creatures at a half (median 0.37 to 0.48; the commonest bytes are
#     128 + 127), on medium ones at about a quarter (0.20 to 0.40); the second ring carries a quarter on soft
#     creatures and nothing on the others; the third ring nothing. Beyond one radius along the limb: nothing.
#   - HOW FAR: per blended joint the farthest vertex still carrying a tenth of the other bone sits 0.45 to 0.6 radii
#     from the joint = 0.2 to 0.3 of that side's bone; a hinge reaches farther AFTER the joint (0.58 radii, 0.34 of
#     the child bone) than before it (0.44, 0.20). Sideways it stops by 2.5 radii from the limb's axis.
#   - Hinge, ball and chain are painted alike within those spreads; a part that never turns is left rigid (78%).
# So the rule is a STYLE the author chooses, per creature or per joint:
RING_STYLES = {                  # amplitude at the joint plane, reach in radii along the limb, mesh rings from the boundary
    'soft':   dict(amp=0.5, reach=1.1, hops=2),      # |x| < 0.28 radii: half and half; to 0.83: a quarter (after rounding to quarters)
    'medium': dict(amp=0.36, reach=0.7, hops=1),     # the touching ring within 0.45 radii: a quarter
    'hard':   dict(amp=0.0, reach=0.0, hops=0)}      # one bone a vertex
RING_KIND = {'ball': (1.0, 1.0), 'chain': (1.0, 1.0), 'hinge': (0.85, 1.3), 'rigid': (0.0, 0.0)}   # reach x (before the joint, after it)
RING_LEN_CAP = 0.4               # the blend never reaches farther than this share of the bone on that side
RING_LATERAL = 2.0               # nor to vertices farther than this many radii from the limb's axis


def weld_graph(V, F):
    """Welded vertex ids (vertices on one whole-unit position are one), and neighbours per welded id."""
    import collections
    _, inv = np.unique(np.rint(np.asarray(V)).astype(int), axis=0, return_inverse=True); inv = inv.ravel()
    nb = collections.defaultdict(set)
    for f in inv[np.asarray(F)]:
        x, y, z = int(f[0]), int(f[1]), int(f[2]); nb[x] |= {y, z}; nb[y] |= {x, z}; nb[z] |= {x, y}
    return inv, nb


def joint_kinds(sk, owned, angle=None, axis_share=None, declared=None, min_angle=8.0):
    """A kind per bone for the joint at its head: 'rigid' | 'chain' | 'hinge' | 'ball'.
    declared {bone: kind} wins. With motion known (angle = the bone's largest turn against its parent over the
    creature's sequences, degrees; axis_share = the share of that turning about one axis): rigid under min_angle,
    hinge when axis_share >= 0.9. From the tree alone: chain = a run of 4 or more single-child bones; otherwise ball."""
    memo = {}
    def sub(b):
        if b not in memo: memo[b] = [b] + [y for c in sk.children[b] for y in sub(c)]
        return memo[b]
    wk = lambda i: [c for c in sk.children[i] if any(owned[d] for d in sub(c))]
    out = {}
    for c in range(sk.n):
        if declared and c in declared: out[c] = declared[c]; continue
        if angle is not None and angle[c] < min_angle: out[c] = 'rigid'; continue
        n = 1; q = c
        while len(wk(q)) == 1: q = wk(q)[0]; n += 1
        q = c
        while sk.parent[q] >= 0 and len(wk(sk.parent[q])) == 1 and owned[sk.parent[q]]: q = sk.parent[q]; n += 1
        if n >= 4 and sk.parent[c] >= 0 and len(wk(sk.parent[c])) == 1: out[c] = 'chain'
        elif axis_share is not None and axis_share[c] >= 0.9: out[c] = 'hinge'
        else: out[c] = 'ball'
    return out


def paint_rings(sk, V, F, dom, kinds=None, style='soft', styles=None, floor=0.06, report=None, len_cap=RING_LEN_CAP, lateral=RING_LATERAL,
                centre='joint', min_boundary=3, amp=None, reach=None, hops=None, webs=True, web_min=0.45, web_share=0.25):
    """THE RING RULE. Given a one-bone-per-vertex assignment `dom`, paint the shared rings at every joint.
    For each bone c with parent p whose two regions touch on the (welded) mesh with at least `min_boundary` vertices:
      axis    the bisector of the limb's direction before and after the joint (parent joint -> joint -> the child's
              own continuation), through the joint;
      radius  the median distance from that axis of the vertices on the boundary between the two regions;
      x       a vertex's distance from the joint plane along the axis, in radii;
      share   of the bone ACROSS the joint = amp x (1 - |x| / reach), for vertices within `hops` mesh rings of the
              boundary and within `lateral` radii of the axis; reach is scaled by the joint's kind (a hinge reaches
              0.85 of it before the joint and 1.3 after) and capped at `len_cap` of the bone on that side.
    style: 'soft' | 'medium' | 'hard' (RING_STYLES), for the whole creature; styles {bone: style} overrides it for the
    joint at that bone's head; amp / reach / hops override the style's numbers. kinds {bone: kind} (joint_kinds).
    A vertex keeps its own bone as the heaviest (the cross shares together never exceed a half); shares under `floor`
    are dropped. Returns dense W (rows sum to 1); the caller rounds to quarters and makes the bytes."""
    V = np.asarray(V, float); dom = np.asarray(dom); nv = len(V)
    inv, nb = weld_graph(V, F); rep = {}; members = {}
    for i, k in enumerate(inv): rep.setdefault(int(k), i); members.setdefault(int(k), []).append(i)
    domw = {k: int(dom[v]) for k, v in rep.items()}; H = sk.head()
    owned = np.zeros(sk.n, bool); owned[list(set(domw.values()))] = True
    kinds = kinds or joint_kinds(sk, owned)
    region = {}
    for k, d in domw.items(): region.setdefault(d, set()).add(k)
    cross = [dict() for _ in range(nv)]; joints = []
    memo = {}
    def sub(b):
        if b not in memo: memo[b] = [b] + [y for c in sk.children[b] for y in sub(c)]
        return memo[b]
    for c in range(sk.n):
        p = sk.parent[c]
        if p < 0 or not owned[c] or not owned[p]: continue
        st = dict(RING_STYLES[(styles or {}).get(c, style)])
        if amp is not None: st['amp'] = amp
        if reach is not None: st['reach'] = reach
        if hops is not None: st['hops'] = hops
        kind = kinds.get(c, 'ball'); kb, ka = RING_KIND[kind]
        if st['amp'] <= 0 or st['hops'] <= 0 or (kb <= 0 and ka <= 0): continue
        Pk, Ck = region[p], region[c]
        b_c = {k for k in Ck if nb[k] & Pk}; b_p = {k for k in Pk if nb[k] & Ck}
        if len(b_c) + len(b_p) < min_boundary: continue
        ring = {}
        for side, start, reg in ((1, b_c, Ck), (-1, b_p, Pk)):
            cur = set(start); seen = set(start); h = 1
            while cur and h <= st['hops']:
                for v in cur: ring[v] = side * h
                nxt = set()
                for v in cur: nxt |= (nb[v] & reg) - seen
                seen |= nxt; cur = nxt; h += 1
        kids = [q for q in sk.children[c] if any(owned[d] for d in sub(q))]
        kid = max(kids, key=lambda q: len(sub(q))) if kids else None
        ap = H[c] - H[p]; cen = V[[rep[k] for k in Ck]].mean(0)
        ac = (H[kid] - H[c]) if kid is not None and np.linalg.norm(H[kid] - H[c]) > 1 else (cen - H[c])
        ap = ap / max(np.linalg.norm(ap), 1e-9); ac = ac / max(np.linalg.norm(ac), 1e-9)
        ax = ap + ac if np.linalg.norm(H[c] - H[p]) > 1 and ap @ ac > -0.5 else ac
        ax = ax / max(np.linalg.norm(ax), 1e-9)
        bv = V[[rep[k] for k in (b_c | b_p)]] - H[c]; rad = max(float(np.median(np.linalg.norm(bv - np.outer(bv @ ax, ax), axis=1))), 2.0)
        len_p = float(np.linalg.norm(H[c] - H[p])); len_c = float(np.linalg.norm(H[kid] - H[c])) if kid is not None else 2 * float(np.linalg.norm(cen - H[c]))
        before = min(st['reach'] * kb, len_cap * len_p / rad) if len_p > 1 else st['reach'] * kb
        after = min(st['reach'] * ka, len_cap * max(len_c, 1) / rad)
        x0 = float(np.median(bv @ ax)) / rad if centre == 'boundary' else 0.0                      # where the two regions actually meet
        n = 0
        for k, r in ring.items():
            dv = V[rep[k]] - H[c]; x = float(dv @ ax) / rad - x0
            if lateral and float(np.linalg.norm(dv - (dv @ ax) * ax)) / rad > lateral: continue
            far = before if x < 0 else after
            other = st['amp'] * max(0.0, 1 - abs(x) / max(far, 1e-6))
            if other < floor: continue
            n += 1; b = p if r > 0 else c
            for v in members[k]: cross[v][b] = max(cross[v].get(b, 0), other)
        joints.append(dict(bone=c, parent=p, kind=kind, radius=rad, boundary=len(b_c) + len(b_p), painted=n))
    nweb = 0
    if webs and RING_STYLES[style]['amp'] > 0:
        # WEBS: where the regions of two bones that are NOT parent and child touch (a membrane between two fingers of a
        # wing, skin between two limbs): measured (ring_webs.py), a vertex carries about a quarter of such a
        # neighbouring bone when that bone owns 45% or more of its mesh neighbourhood (median 0.15 to 0.25 on soft and
        # medium creatures), and nothing when it owns under a quarter (median 0).
        seg = segments(sk) if webs == 'distance' else None
        for k, v0 in rep.items():
            a = domw[k]; cnt = {}
            for n_ in nb[k]: cnt[domw[n_]] = cnt.get(domw[n_], 0) + 1
            tot = sum(cnt.values())
            for b, c in cnt.items():
                if b == a or sk.parent[a] == b or sk.parent[b] == a: continue
                fr = c / tot
                if fr < web_min: continue
                if webs == 'distance':                      # by the two bones' distances: the nearer bone takes more, never over a half
                    da = min(point_segment(V[v0:v0 + 1], p0, p1)[0][0] for bb, p0, p1 in seg if bb == a); db = min(point_segment(V[v0:v0 + 1], p0, p1)[0][0] for bb, p0, p1 in seg if bb == b)
                    s = min(0.5, da / max(da + db, 1e-9))
                else: s = web_share
                nweb += 1
                for v in members[k]: cross[v][b] = max(cross[v].get(b, 0), s)
    W = np.zeros((nv, sk.n))
    for v in range(nv):
        tot = sum(cross[v].values())
        sc = min(1.0, 0.5 / tot) if tot > 0.5 else 1.0                 # the vertex's own bone keeps at least half
        for b, s in cross[v].items(): W[v, b] = s * sc
        W[v, dom[v]] += 1 - W[v].sum()
    if report is not None: report.extend(joints); report.append(dict(webs=nweb))
    return W


def smooth(W, adj, rounds=1, keep=0.5):
    """Each vertex's row becomes keep x itself + (1 - keep) x the mean of its neighbours' rows."""
    W = W.copy()
    for _ in range(rounds):
        N = np.stack([W[a].mean(0) if a else W[i] for i, a in enumerate(adj)])
        W = keep * W + (1 - keep) * N
    return W
