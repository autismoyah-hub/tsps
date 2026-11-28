"""WEBS: what real creatures paint where the regions of two bones that are NOT parent and child touch on the mesh
(a membrane between two wing fingers, skin between a limb and the trunk's next bone, two tentacles side by side).
usage: ring_webs.py [--json OUT]
For every (welded) vertex v owned by bone a with a mesh neighbour owned by another bone b: the share of b on v, by
the relation of a and b (parent-child | siblings | other), by the creature's softness, and by the share of v's
neighbours that b owns. Also where the other-relation blends that exist are NOT at such a touching boundary."""
import sys, json, pathlib, collections
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
import riglib as R, wlib
import ring_census as RC
L, B, RM = R.L, R.B, R.RM
rows = []
def cls(one): return 'soft' if one < .4 else 'medium' if one < .65 else 'hard'
for x in RC.creatures():
    sk = RM.Skeleton(B.decode_base(L.base_bytes(x['base'])))
    def tdist(a, b):
        pa = []; q = a
        while q >= 0: pa.append(q); q = sk.parent[q]
        q = b; d = 0
        while q >= 0:
            if q in pa: return d + pa.index(q)
            q = sk.parent[q]; d += 1
        return 99
    for mid in x['models']:
        m = B.decode_model(L.model_bytes(mid))
        if m['bone_weights'] is None: continue
        V = np.array(m['vertices'], float); F = np.array(m['faces']); Wr = RM.weight_matrix(m['bone_weights'], sk.n); dom = Wr.argmax(1)
        inv, nb = wlib.weld_graph(V, F); rep = {}
        for i, k in enumerate(inv): rep.setdefault(int(k), i)
        for k, v in rep.items():
            a = int(dom[v]); cnt = collections.Counter(int(dom[rep[n]]) for n in nb[k]); tot = sum(cnt.values())
            for b, c in cnt.items():
                if b == a: continue
                d = tdist(a, b); rel = 'parent-child' if d == 1 else 'siblings' if (d == 2 and sk.parent[a] == sk.parent[b]) else 'other'
                rows.append((cls(x['one']), rel, c / tot, float(Wr[v, b]), d, x['npc']))
print(f'{len(rows)} (vertex, neighbouring bone) pairs on {len(set(r[5] for r in rows))} creatures')
print('share of the neighbouring bone on the vertex: MEAN / MEDIAN / share of vertices over 0.1 / n; by how much of the vertex\'s neighbourhood that bone owns')
for style in ('soft', 'medium', 'hard'):
    print(f'  {style}:')
    for rel in ('parent-child', 'siblings', 'other'):
        line = []
        for lo, hi in ((0, .26), (.26, .45), (.45, .7), (.7, 1.01)):
            s = np.array([r[3] for r in rows if r[0] == style and r[1] == rel and lo <= r[2] < hi])
            if len(s) >= 15: line.append(f'{lo}-{hi}: {s.mean():.2f}/{np.median(s):.2f}/{np.mean(s > .1):.2f}/{len(s)}')
        print(f'    {rel:12s} neighbours owned by the other bone ' + '  '.join(line))
if '--json' in sys.argv: pathlib.Path(sys.argv[sys.argv.index('--json') + 1]).write_text(json.dumps(rows))
