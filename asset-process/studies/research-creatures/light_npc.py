"""NPC lighting: a Python port of the client's Model::normals / Model::light with the NPC parameters, checked
against the client's own output, plus joint statistics of a rest mesh.

The client lights an NPC ONCE, in the rest pose, before animation and before the definition's scale
(rust-client/src/world_actors.rs:457): light(64 + ambient, 850 + contrast, [-30, -50, -30]), contrast = stored x 5.

usage:
  light_npc.py dump  ENTRY OUT.txt [seq=ID:FRAME]   run creature_light.exe (ENTRY = npc:ID | file:PATH[;ambient=A;contrast=C])
  light_npc.py check OUT.txt [...]                  port against the client's lit colours, corner by corner
  light_npc.py relit OUT.txt                        rest colours (what is drawn) against what a re-light of the POSED mesh
                                                    would give (file must have been dumped with seq=)
  light_npc.py joints OUT.txt [...]                 how the rest mesh is built at label boundaries
as a module:
  V, L, F, amb, con, posed, relit = read(path)      F rows: (a, b, c, colour, render, alpha, texture, (c0, c1, c2))
  lit = light(V, F, amb, con)                       per face (c0, c1, c2) exactly as Model::light returns them:
        smooth: three lit 16-bit colours; flat: (c, 0, -1); alpha 254 untextured: (128, 0, -1); not drawn: (0, 0, -2)
  vertex_light(V, F, amb, con)                      per vertex: the brightness number (before paint), or None
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, math, subprocess, pathlib, collections
EXE = _AP.fix(r'<CLIENT_BIN>\creature_light.exe')
CACHE = _AP.fix(r'<OSRS_CACHE_239>')
DIRECTION = (-30, -50, -30)

def tdiv(a, b):
    q = abs(a) // abs(b); return q if (a >= 0) == (b > 0) else -q
def read(path):
    V, L, F, posed, relit = [], [], [], [], []; amb = con = None
    for line in pathlib.Path(path).read_text().splitlines():
        p = line.split()
        if not p or p[0] == '#': continue
        if p[0] == 'h': amb, con = int(p[1]), int(p[2])
        elif p[0] == 'v': V.append((int(p[1]), int(p[2]), int(p[3]))); L.append(int(p[4]))
        elif p[0] == 'f': F.append((int(p[1]), int(p[2]), int(p[3]), int(p[4]), int(p[5]), int(p[6]), int(p[7]), (int(p[8]), int(p[9]), int(p[10]))))
        elif p[0] == 'p': posed.append((int(p[1]), int(p[2]), int(p[3])))
        elif p[0] == 'r': relit.append((int(p[1]), int(p[2]), int(p[3])))
    return V, L, F, amb, con, posed, relit
def normals(V, F):
    vert = [[0, 0, 0, 0] for _ in V]; face = [(0, 0, 0)] * len(F)
    for i, f in enumerate(F):
        a, b, c = V[f[0]], V[f[1]], V[f[2]]
        d = [b[k] - a[k] for k in range(3)]; e = [c[k] - a[k] for k in range(3)]
        n = [d[1] * e[2] - e[1] * d[2], d[2] * e[0] - e[2] * d[0], d[0] * e[1] - e[0] * d[1]]
        while any(not -8192 <= v <= 8192 for v in n): n = [v >> 1 for v in n]
        length = max(int(math.sqrt(sum(v * v for v in n))), 1)
        n = [tdiv(v * 256, length) for v in n]
        if f[4] == 0:
            for v in f[:3]:
                for k in range(3): vert[v][k] += n[k]
                vert[v][3] += 1
        elif f[4] == 1: face[i] = tuple(n)
    return vert, face
def denominator(con): return (con * int(math.sqrt(sum(d * d for d in DIRECTION)))) >> 8
def vertex_light(V, F, amb, con):
    vert, _ = normals(V, F); den = denominator(con)
    return [amb + tdiv(sum(n[k] * DIRECTION[k] for k in range(3)), den * n[3]) if n[3] else None for n in vert]
def light(V, F, amb, con):
    vert, face = normals(V, F); den = denominator(con); out = []
    for i, f in enumerate(F):
        colour, kind, alpha, tex = f[3], f[4], f[5], f[6]
        if alpha == 254: kind = 3
        elif alpha == 255: kind = 2
        textured = tex != -1
        def shade(b):
            if textured: return max(2, min(126, b))
            return (colour & 65408) + max(2, min(126, ((colour & 127) * b) >> 7))
        if kind == 0:
            cs = []
            for v in f[:3]:
                n = vert[v]; cs.append(shade(amb + tdiv(sum(n[k] * DIRECTION[k] for k in range(3)), den * n[3])))
            out.append(tuple(cs))
        elif kind == 1:
            out.append((shade(amb + tdiv(sum(face[i][k] * DIRECTION[k] for k in range(3)), den + tdiv(den, 2))), 0, -1))
        elif kind == 3 and not textured: out.append((128, 0, -1))
        else: out.append((0, 0, -2))
    return out

def check(path):
    V, L, F, amb, con, _, _ = read(path); mine = light(V, F, amb, con)
    faces = sum(1 for f, m in zip(F, mine) if f[7] == m); corners = bad = worst = 0
    for f, m in zip(F, mine):
        n = 1 if m[2] == -1 else 0 if m[2] == -2 else 3
        for k in range(n):
            corners += 1
            if f[7][k] != m[k]: bad += 1; worst = max(worst, abs((f[7][k] & 127) - (m[k] & 127)))
    kinds = collections.Counter(f[4] for f in F); al = sum(1 for f in F if f[5]); tx = sum(1 for f in F if f[6] != -1)
    print(f'{path}: light({amb}, {con}); faces {len(F)}: {faces} identical ({100 * faces / len(F):.2f}%); lit corners {corners}: {corners - bad} identical, '
          f'{bad} differ (worst {worst} lightness steps); render types {dict(sorted(kinds.items()))}; alpha faces {al}; textured {tx}; not drawn {sum(1 for m in mine if m[2] == -2)}')
    return faces == len(F)

def relit_report(path):
    V, L, F, amb, con, posed, relit = read(path)
    if not relit: raise SystemExit('dump with seq=ID:FRAME first')
    moved = sum(1 for a, b in zip(V, posed) if a != b)
    port = light(posed, F, amb, con)
    same_port = sum(1 for a, b in zip(port, relit) if a == b)
    n = d = 0; big = 0; deltas = []
    for f, r in zip(F, relit):
        k = 1 if r[2] == -1 else 0 if r[2] == -2 else 3
        for j in range(k):
            n += 1; dl = abs((f[7][j] & 127) - (r[j] & 127)); deltas.append(dl)
            if dl: d += 1
            if dl >= 8: big += 1
    deltas.sort()
    print(f'{path}: {moved} of {len(V)} vertices moved by the frame. Drawn colours are the REST colours. A re-light of the posed mesh would change '
          f'{d} of {n} lit corners ({100 * d / n:.1f}%), {big} by 8 or more lightness steps ({100 * big / n:.1f}%); median change {deltas[len(deltas) // 2]}, '
          f'90th centile {deltas[9 * len(deltas) // 10]}, largest {deltas[-1]}. (port on posed mesh = client relit on {same_port} of {len(F)} faces)')

def joints(path):
    V, L, F, amb, con, _, _ = read(path)
    nf = len(F)
    multi = sum(1 for f in F if len({L[v] for v in f[:3]}) > 1)
    pos = collections.defaultdict(list)
    for i, v in enumerate(V): pos[v].append(i)
    split = sum(len(g) for g in pos.values() if len(g) > 1)
    split_cross = sum(len(g) for g in pos.values() if len(g) > 1 and len({L[i] for i in g}) > 1)
    # components by shared vertex index
    parent = list(range(len(V)))
    def find(x):
        while parent[x] != x: parent[x] = parent[parent[x]]; x = parent[x]
        return x
    for f in F:
        a = find(f[0])
        for v in f[1:3]: parent[find(v)] = a
    used = {v for f in F for v in f[:3]}
    comp = collections.defaultdict(list)
    for v in used: comp[find(v)].append(v)
    edges = collections.Counter()
    for f in F:
        for a, b in ((f[0], f[1]), (f[1], f[2]), (f[2], f[0])): edges[(min(a, b), max(a, b))] += 1
    open_edges = [e for e, n in edges.items() if n == 1]
    # open loops: connected groups of boundary edges
    adj = collections.defaultdict(set)
    for a, b in open_edges: adj[a].add(b); adj[b].add(a)
    seen = set(); loops = []
    for s in adj:
        if s in seen: continue
        stack = [s]; g = []
        while stack:
            x = stack.pop()
            if x in seen: continue
            seen.add(x); g.append(x); stack.extend(adj[x] - seen)
        loops.append(g)
    boxes = {}
    for c, vs in comp.items():
        boxes[c] = [(min(V[v][k] for v in vs), max(V[v][k] for v in vs)) for k in range(3)]
    tucked = 0; loop_multi = 0
    for g in loops:
        c = find(g[0]); cen = [sum(V[v][k] for v in g) / len(g) for k in range(3)]
        if any(o != c and all(b[k][0] <= cen[k] <= b[k][1] for k in range(3)) for o, b in boxes.items()): tucked += 1
        if len({L[v] for v in g}) > 1: loop_multi += 1
    closed = sum(1 for c, vs in comp.items() if not any(find(a) == c for a, b in open_edges))
    labels = sorted(set(L[v] for v in used))
    # boundary vertices between labels: a vertex whose faces reach another label
    reach = collections.defaultdict(set)
    for f in F:
        ls = {L[v] for v in f[:3]}
        for v in f[:3]: reach[v] |= ls
    ring = sum(1 for v in used if len(reach[v]) > 1)
    comp_labels = [len({L[v] for v in vs}) for vs in comp.values()]
    print(f'{path}: {len(used)} vertices {nf} faces, {len(labels)} labels')
    print(f'   faces spanning two or more labels (welded bridge across a joint): {multi} ({100 * multi / nf:.1f}%); vertices touching such a face: {ring} ({100 * ring / len(used):.1f}%)')
    print(f'   vertices sharing a position with another vertex (split): {split} ({100 * split / len(V):.1f}%), of which with DIFFERENT labels: {split_cross}')
    print(f'   connected pieces: {len(comp)} (closed, no open edge: {closed}); pieces carrying one label only: {sum(1 for n in comp_labels if n == 1)}; '
          f'largest piece {max(len(v) for v in comp.values())} vertices')
    print(f'   open edges: {len(open_edges)} in {len(loops)} loops; loops whose centre lies inside another piece\'s box (tucked end, proxy): {tucked}; loops spanning labels: {loop_multi}')

if __name__ == '__main__':
    a = sys.argv
    if len(a) < 3: raise SystemExit(__doc__)
    if a[1] == 'dump':
        r = subprocess.run([EXE, CACHE] + a[2:], capture_output=True, text=True); print((r.stdout + r.stderr).strip()); raise SystemExit(r.returncode)
    if a[1] == 'check':
        ok = all([check(p) for p in a[2:]]); raise SystemExit(0 if ok else 1)
    if a[1] == 'relit': relit_report(a[2])
    if a[1] == 'joints':
        for p in a[2:]: joints(p)
