"""research-clean library: load a creature as the client assembles it, light it exactly as the client does (once, at
rest, by vertex vote), and draw it with an own z-buffer so that every pixel knows its face.

    c = load('npc:12812')  or  load('def:PATH.json')  or  load(PATH.json)  or  load('12812')
        -> dict: V (float, cache coords: x left-right, y DOWN, z along the body, front = -z), L labels, F faces, C paint
           (16-bit HSL after recolour), T render type, A alpha, name, ambient, contrast (as the client uses them),
           LN (light number per vertex, None-> -1 where no smooth face votes), LC (lit lightness per face corner [nf, 3]),
           drawn (bool per face), area, unit (outward unit normals), cen
    img, ids = view(c, yaw, pitch, ...)   lit picture (Gouraud on lightness, the game's colour table) and face ids

Light: Model::light as ported in research-creatures/light_npc.py (checked there against the client corner by corner);
this is the same arithmetic in NumPy, and `selfcheck()` compares the two on any entry.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, pathlib, json, math, collections
import numpy as np
HERE = pathlib.Path(__file__).resolve().parent
B = HERE.parent
sys.path.insert(0, str(B / 'research-creatures')); sys.path.insert(1, str(B / 'research-npc')); sys.path.insert(2, _AP.fix(r'<ROOT>\author-pack\tools'))
import cmesh, gamecol
cmesh.DATA = HERE / 'data'
TABLE = gamecol.table()
DIRECTION = np.array([-30, -50, -30], np.int64)
BG = (48, 43, 37)
_census = None

def hsl(c): return (int(c) >> 10) & 63, (int(c) >> 7) & 7, int(c) & 127

def census():
    global _census
    if _census is None:
        _census = {}
        for l in (B / 'research-npc' / 'census' / 'npcs.jsonl').read_text(encoding='utf-8').splitlines():
            if l.strip():
                j = json.loads(l); _census[j['id']] = j
    return _census

def norm_entry(e):
    e = str(e)
    if e.isdigit(): return 'npc:' + e
    if e.startswith('npc:') or e.startswith('def:'): return e
    p = pathlib.Path(e)
    if p.is_dir():
        js = [q for q in p.glob('*.json') if 'models' in q.read_text(encoding='utf-8', errors='ignore')[:400]]
        want = [q for q in js if q.stem == p.name] or js
        if not want: raise SystemExit(f'no definition JSON in {p}')
        p = want[0]
    return 'def:' + str(p.resolve())

def _tdiv(a, b):
    """integer division toward zero, elementwise (b > 0)"""
    return np.sign(a) * (np.abs(a) // b)

def light_numbers(V, F, T, amb, con):
    """per-vertex light number (int; -999 where no smooth face votes) and per-face flat light number"""
    Vi = np.round(V).astype(np.int64)
    a, b, c = Vi[F[:, 0]], Vi[F[:, 1]], Vi[F[:, 2]]
    d, e = b - a, c - a
    n = np.stack([d[:, 1] * e[:, 2] - e[:, 1] * d[:, 2], d[:, 2] * e[:, 0] - e[:, 2] * d[:, 0], d[:, 0] * e[:, 1] - e[:, 0] * d[:, 1]], 1)
    for _ in range(40):
        big = (np.abs(n) > 8192).any(1)
        if not big.any(): break
        n[big] >>= 1
    length = np.maximum(np.floor(np.sqrt((n * n).sum(1).astype(float))).astype(np.int64), 1)
    n = _tdiv(n * 256, length[:, None])
    acc = np.zeros((len(V), 3), np.int64); votes = np.zeros(len(V), np.int64)
    sm = T == 0
    for k in range(3):
        np.add.at(acc, F[sm, k], n[sm]); np.add.at(votes, F[sm, k], 1)
    den = (con * 65) >> 8
    dot = acc @ DIRECTION
    ln = np.where(votes > 0, amb + _tdiv(dot, den * np.maximum(votes, 1)), -999)
    flat = amb + _tdiv(n @ DIRECTION, den + den // 2)
    return ln, flat, votes

def load(entry):
    entry = norm_entry(entry)
    m = cmesh.load(entry)
    if entry.startswith('npc:'):
        d = census()[int(entry[4:])]; name = f"{d['name']} {entry[4:]}"; amb, con = d['ambient'], d['contrast']; real = True
    else:
        j = json.loads(pathlib.Path(entry[4:]).read_text()); name = j.get('name', pathlib.Path(entry[4:]).stem)
        amb, con = int(j.get('ambient', 0)), int(j.get('contrast', 0)) * 5; real = False
    V, F, C, T, A = m['V'], m['F'], m['C'], m['T'], m['A']
    area, unit, cen = cmesh.face_geometry(m)
    sign = cmesh.outward_sign(m, area, unit, cen)
    ln, flat, votes = light_numbers(V, F, T, 64 + amb, 850 + con)
    pl = (C & 127).astype(np.int64)
    lc = np.clip((pl[:, None] * ln[F]) >> 7, 2, 126)
    fl = np.clip((pl * flat) >> 7, 2, 126)
    lc[T == 1] = fl[T == 1][:, None]
    drawn = (A != 255) & (T < 2) & (area > 0)
    return dict(entry=entry, name=name, real=real, V=V, L=m['L'], F=F, C=C, T=T, A=A, D=m['D'], ambient=amb, contrast=con, LN=ln, LC=lc, votes=votes,
                drawn=drawn, area=area, unit=unit * sign, cen=cen, mesh=m, sign=sign)

def from_mesh(m, name='mesh', ambient=0, contrast_stored=0):
    """The same dict as load() from a meshio mesh held in memory (one model, no recolour): for scripts that edit a mesh."""
    nf = len(m['faces'])
    mm = dict(V=np.array(m['vertices'], float).reshape(-1, 3), L=np.array(m['vertex_skins'], int), F=np.array(m['faces'], int).reshape(-1, 3), C=np.array(m['colors'], int),
              P=np.array(m['priorities'], int), T=np.array(m['render_types'], int), D=np.array(m['depth'], int), A=np.array(m['alpha'], int), FL=np.array(m.get('face_skins') or [0] * nf, int), head='')
    V, F, C, T, A = mm['V'], mm['F'], mm['C'], mm['T'], mm['A']
    area, unit, cen = cmesh.face_geometry(mm)
    sign = cmesh.outward_sign(mm, area, unit, cen)
    ln, flat, votes = light_numbers(V, F, T, 64 + ambient, 850 + contrast_stored * 5)
    pl = (C & 127).astype(np.int64)
    lc = np.clip((pl[:, None] * ln[F]) >> 7, 2, 126)
    fl = np.clip((pl * flat) >> 7, 2, 126)
    lc[T == 1] = fl[T == 1][:, None]
    drawn = (A != 255) & (T < 2) & (area > 0)
    return dict(entry='mesh:' + name, name=name, real=False, V=V, L=mm['L'], F=F, C=C, T=T, A=A, D=mm['D'], ambient=ambient, contrast=contrast_stored * 5, LN=ln, LC=lc, votes=votes,
                drawn=drawn, area=area, unit=unit * sign, cen=cen, mesh=mm, sign=sign)

def selfcheck(entry):
    """NumPy light against the checked list port (light_npc.light)."""
    import light_npc
    c = load(entry)
    F = [(int(f[0]), int(f[1]), int(f[2]), int(col), int(t), int(al), -1, None) for f, col, t, al in zip(c['F'], c['C'], c['T'], c['A'])]
    V = [tuple(int(round(x)) for x in v) for v in c['V']]
    ref = light_npc.light(V, F, 64 + c['ambient'], 850 + c['contrast'])
    bad = 0; n = 0
    for i, r in enumerate(ref):
        if r[2] == -2: continue
        if r[2] == -1:
            n += 1; bad += int((r[0] & 127) != c['LC'][i, 0]); continue
        for k in range(3): n += 1; bad += int((r[k] & 127) != c['LC'][i, k])
    return n, bad

# ---------------------------------------------------------------------------------------------------- drawing
def camera(yaw, pitch):
    t, p = math.radians(yaw), math.radians(pitch)
    f = np.array([math.sin(t), 0, math.cos(t)]); r = np.array([-math.cos(t), 0, math.sin(t)])
    fwd = f * math.cos(p) + np.array([0, 1, 0]) * math.sin(p); down = np.array([0, 1, 0]) * math.cos(p) - f * math.sin(p)
    return r, down, fwd

def frame(c, yaw, pitch, ppu=None, size=420, box=None, keep=None, pad=8):
    """Screen coordinates of every vertex for the picture `view` makes with the same arguments: X, Y, depth, W, H."""
    V, F = c['V'], c['F']
    r, down, fwd = camera(yaw, pitch)
    P = np.stack([V @ r, V @ down, V @ fwd], 1)
    if keep is None: keep = np.nonzero(c['drawn'])[0]
    if box is None:
        used = np.unique(F[keep]); lo, hi = P[used].min(0), P[used].max(0)
    else:
        corners = np.array([[box[(k >> 0) & 1][0], box[(k >> 1) & 1][1], box[(k >> 2) & 1][2]] for k in range(8)], float)
        Pc = np.stack([corners @ r, corners @ down, corners @ fwd], 1); lo, hi = Pc.min(0), Pc.max(0)
    s = ppu if ppu else (size - 2 * pad) / max(hi[0] - lo[0], hi[1] - lo[1], 1)
    W, H = int((hi[0] - lo[0]) * s) + 2 * pad, int((hi[1] - lo[1]) * s) + 2 * pad
    return (P[:, 0] - lo[0]) * s + pad, (P[:, 1] - lo[1]) * s + pad, P[:, 2], W, H

def view(c, yaw=0.0, pitch=10.0, ppu=None, size=420, box=None, face_rgb=None, tint=None, bg=BG, faces=None, pad=8, light=True):
    """Lit picture by an own z-buffer. yaw 0 = from the front (-z side), 90 = the creature's -x flank ... pitch = degrees
    the camera looks down. box = (lo, hi) in model coordinates to frame (default: everything drawn).
    face_rgb: per-face RGB replacing the lit colour (array [nf, 3]); tint: dict face -> RGB mixed 65% over the lit colour.
    Returns img [H, W, 3] uint8 and ids [H, W] (face index or -1)."""
    V, F = c['V'], c['F']
    keep = np.nonzero(c['drawn'])[0] if faces is None else np.asarray(faces)
    X, Y, Z, W, H = frame(c, yaw, pitch, ppu, size, box, keep, pad)
    img = np.zeros((H, W, 3), np.uint8); img[:] = bg; zb = np.full((H, W), np.inf); ids = np.full((H, W), -1, int)
    LC = c['LC']; C = c['C']; A = c['A']
    for k in keep:
        tri = F[k]
        x, y, z = X[tri], Y[tri], Z[tri]
        x0, x1 = max(int(math.floor(x.min())), 0), min(int(math.ceil(x.max())), W - 1); y0, y1 = max(int(math.floor(y.min())), 0), min(int(math.ceil(y.max())), H - 1)
        if x1 < x0 or y1 < y0: continue
        den = (y[1] - y[2]) * (x[0] - x[2]) + (x[2] - x[1]) * (y[0] - y[2])
        if abs(den) < 1e-9: continue
        gx, gy = np.meshgrid(np.arange(x0, x1 + 1) + 0.5, np.arange(y0, y1 + 1) + 0.5)
        a = ((y[1] - y[2]) * (gx - x[2]) + (x[2] - x[1]) * (gy - y[2])) / den
        b = ((y[2] - y[0]) * (gx - x[2]) + (x[0] - x[2]) * (gy - y[2])) / den
        w = 1 - a - b
        zz = a * z[0] + b * z[1] + w * z[2]
        ins = (a >= -1e-6) & (b >= -1e-6) & (w >= -1e-6) & (zz < zb[y0:y1 + 1, x0:x1 + 1])
        if not ins.any(): continue
        if face_rgb is not None: col = np.broadcast_to(np.asarray(face_rgb[k], np.uint8), ins.shape + (3,))[ins]
        else:
            if light: l = np.clip(np.rint(a * LC[k, 0] + b * LC[k, 1] + w * LC[k, 2]), 0, 127).astype(int)[ins]
            else: l = np.full(int(ins.sum()), int(C[k]) & 127)
            col = TABLE[(int(C[k]) & 0xFF80) + l].astype(np.uint8)
            if A[k] not in (0, 255):            # translucent: mix over what is there (rough; never measured)
                al = (255 - int(A[k])) / 255.0; col = (col * al + img[y0:y1 + 1, x0:x1 + 1][ins] * (1 - al)).astype(np.uint8)
            if tint is not None and k in tint: col = (col * 0.3 + np.array(tint[k]) * 0.7).astype(np.uint8)
        zb[y0:y1 + 1, x0:x1 + 1][ins] = zz[ins]; img[y0:y1 + 1, x0:x1 + 1][ins] = col; ids[y0:y1 + 1, x0:x1 + 1][ins] = k
    return img, ids

# ---------------------------------------------------------------------------------------------------- topology
def topology(c):
    """Edges of the DRAWN opaque-or-not skin: e_a, e_b (faces), e_v (vertices), e_len, e_dih (signed degrees: + convex,
    - concave), open edges, vertex -> faces, face adjacency."""
    if 'e_a' in c: return c
    F = c['F']; keep = np.nonzero(c['drawn'])[0]
    edges = collections.defaultdict(list)
    for i in keep:
        f = F[i]
        for a, b in ((f[0], f[1]), (f[1], f[2]), (f[2], f[0])): edges[(min(a, b), max(a, b))].append(i)
    e2 = [(k, v) for k, v in edges.items() if len(v) == 2]
    c['e_a'] = np.array([v[0] for _, v in e2], int); c['e_b'] = np.array([v[1] for _, v in e2], int)
    c['e_v'] = np.array([k for k, _ in e2], int).reshape(-1, 2)
    V = c['V']; ev = c['e_v']
    c['e_len'] = np.linalg.norm(V[ev[:, 0]] - V[ev[:, 1]], axis=1)
    ua, ub = c['unit'][c['e_a']], c['unit'][c['e_b']]
    ang = np.degrees(np.arccos(np.clip((ua * ub).sum(1), -1, 1)))
    convex = ((c['cen'][c['e_b']] - c['cen'][c['e_a']]) * ua).sum(1) < 0
    c['e_dih'] = np.where(convex, ang, -ang)
    c['open'] = [k for k, v in edges.items() if len(v) == 1]
    c['nonmanifold'] = [k for k, v in edges.items() if len(v) > 2]
    vf = collections.defaultdict(list)
    for i in keep:
        for v in F[i]: vf[int(v)].append(int(i))
    c['vf'] = vf
    adj = collections.defaultdict(list)
    for k, (a, b) in enumerate(zip(c['e_a'], c['e_b'])): adj[int(a)].append((int(b), k)); adj[int(b)].append((int(a), k))
    c['adj'] = adj
    vv = collections.defaultdict(set)
    for (a, b) in edges: vv[int(a)].add(int(b)); vv[int(b)].add(int(a))
    c['vv'] = vv
    # shells by shared vertices
    parent = {}
    def find(x):
        while parent.setdefault(x, x) != x: parent[x] = parent[parent[x]]; x = parent[x]
        return x
    for i in keep:
        f = F[i]; a = find(int(f[0]))
        for v in f[1:]: parent[find(int(v))] = a
    c['shell'] = {int(i): find(int(F[i][0])) for i in keep}
    return c

def grey(rgb): return np.asarray(rgb, float) @ np.array([0.299, 0.587, 0.114])
