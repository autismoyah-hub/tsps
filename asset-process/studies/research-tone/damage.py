"""Step 6: what does the gate's TONE line read on the real boss with small, KNOWN damage? (Teacher-side.)
usage: damage.py NPC_ID
  ceiling       the real boss through our encoder: must read 0
  light+K       every painted lightness K steps up (K = 1, 2, 4)
  islandsP      P percent of the small contrasting islands repainted in their largest neighbour's colour (P = 10, 50, 100)
  resplit       every inner diagonal between two faces of the SAME paint flipped (same vertices, same paint, other split)
  resplit-all   every flippable inner diagonal flipped, paint boundaries included (each face keeps its colour)
  jitterN       every vertex moved up to N units on each axis (N = 1, 2, 3, 6), paint untouched
"""
import sys, pathlib; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import json, random, collections
import numpy as np
import tl
def regions(a):
    V, F, C = a['V'], a['F'], a['C']; dr = tl.drawn(a); area = np.linalg.norm(np.cross(V[F[:, 1]] - V[F[:, 0]], V[F[:, 2]] - V[F[:, 0]]), axis=1) / 2
    parent = list(range(len(F)))
    def find(i):
        while parent[i] != i: parent[i] = parent[parent[i]]; i = parent[i]
        return i
    edges = collections.defaultdict(list)
    for i in np.nonzero(dr)[0]:
        f = F[i]
        for x, y in ((f[0], f[1]), (f[1], f[2]), (f[2], f[0])): edges[(min(x, y), max(x, y))].append(i)
    for v in edges.values():
        for j in v[1:]:
            if C[v[0]] == C[j]: parent[find(j)] = find(v[0])
    root = np.array([find(i) for i in range(len(F))]); ra = collections.Counter()
    for i in np.nonzero(dr)[0]: ra[root[i]] += area[i]
    neigh = collections.defaultdict(collections.Counter)
    for (x, y), v in edges.items():
        el = np.linalg.norm(V[x] - V[y])
        for i in v:
            for j in v:
                if root[i] != root[j]: neigh[root[i]][root[j]] += el
    g = lambda c: float(tl.TABLE[int(c)] @ tl.GREY); tot = area[dr].sum(); isl = []
    for r, ar in ra.items():
        if ar > .012 * tot or not neigh[r]: continue
        big = max(neigh[r].items(), key=lambda kv: kv[1])[0]
        if abs(g(C[r]) - g(C[big])) >= 20: isl.append((r, big))
    return root, isl
def flip_all(m, same_paint=True):
    F = [list(f) for f in m['faces']]; V = np.array(m['vertices'], float); e = collections.defaultdict(list)
    for i, f in enumerate(F):
        for j in range(3): e[(f[j], f[(j + 1) % 3])].append(i)
    und = set(tuple(sorted(k)) for k in e); dirty = set(); n = 0
    keys = [k for k in ('alpha', 'render_types', 'face_skins', 'depth', 'priorities') if m.get(k)] + (['colors'] if same_paint else [])
    def nrm(t): x = np.cross(V[t[1]] - V[t[0]], V[t[2]] - V[t[0]]); l = np.linalg.norm(x); return x / max(l, 1e-9), l
    for (a_, b_), fl in list(e.items()):
        if a_ > b_ or len(fl) != 1 or len(e.get((b_, a_), [])) != 1: continue
        f1, f2 = fl[0], e[(b_, a_)][0]
        if f1 in dirty or f2 in dirty or any(m[k][f1] != m[k][f2] for k in keys) or m['alpha'][f1] == 255: continue
        c_ = [v for v in F[f1] if v not in (a_, b_)]; d_ = [v for v in F[f2] if v not in (a_, b_)]
        if len(c_) != 1 or len(d_) != 1 or c_[0] == d_[0] or tuple(sorted((c_[0], d_[0]))) in und: continue
        c_, d_ = c_[0], d_[0]; n1, l1 = nrm(F[f1]); n2, l2 = nrm(F[f2]); base = n1 + n2; base /= max(np.linalg.norm(base), 1e-9)
        t1, t2 = [a_, d_, c_], [d_, b_, c_]; m1, k1 = nrm(t1); m2, k2 = nrm(t2)
        if k1 < 2 or k2 < 2 or m1 @ base < 0.5 or m2 @ base < 0.5: continue
        F[f1], F[f2] = t1, t2; dirty.update((f1, f2)); und.add(tuple(sorted((c_, d_)))); n += 1
    mm = tl.clone(m); mm['faces'] = [tuple(f) for f in F]; return mm, n
def run(npc):
    m, kv = tl.real_mesh(npc); d = tl.real_def(npc); a = tl.arrays(m); out = {}
    def g(tag, mm, note=''):
        p = tl.write_variant(f'{npc}-{tag}', mm, d, 'damage'); v, vals, rows = tl.gate(npc, p, f'damage-{npc}-{tag}', tl.RENDER / f'research-tone-damage-{npc}-{tag}')
        tl.show_tones(f'{npc} {tag} {note}', v, vals); out[tag] = dict(verdict=v, tones=tl.tone_lines(vals), note=note, colour_distance=vals.get('colour distance'), outline=[vals.get(f'rest outline, {k}') for k in ('side', 'front', 'top')])
    g('ceiling', tl.clone(m))
    for k in (1, 2, 4):
        mm = tl.clone(m); mm['colors'] = [(c & ~127) | max(2, min(126, (c & 127) + k)) for c in m['colors']]; g(f'light+{k}', mm)
    root, isl = regions(a); rng = random.Random(239); order = list(isl); rng.shuffle(order)
    for pct in (10, 50, 100):
        mm = tl.clone(m); take = order[:max(1, round(len(order) * pct / 100))]; col = np.array(m['colors'])
        for r, big in take: col[root == r] = a['C'][big]
        mm['colors'] = [int(c) for c in col]; g(f'islands{pct}', mm, f'({len(take)} of {len(order)} islands removed)')
    mm, n = flip_all(m, True); g('resplit', mm, f'({n} diagonals flipped, same paint both sides)')
    mm, n = flip_all(m, False); g('resplit-all', mm, f'({n} diagonals flipped)')
    for jit in (1, 2, 3, 6):
        rng = random.Random(239); mm = tl.clone(m); mm['vertices'] = [tuple(c + rng.randint(-jit, jit) for c in v) for v in m['vertices']]; g(f'jitter{jit}', mm)
    (tl.WORK / 'damage').mkdir(exist_ok=True); (tl.WORK / 'damage' / f'{npc}.json').write_text(json.dumps(out, indent=1))
if __name__ == '__main__':
    for n in sys.argv[1:]: run(int(n))
