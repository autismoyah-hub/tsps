"""Score re-assigned weights against the real creature's, in MOTION, vertex by vertex (no drawing: fast, exact).

usage: rig_wscore.py WORKDIR OURS_MODEL.bin [--all] [--worst N] [--json OUT]
Both meshes must be the same vertices in the same order (the real mesh with other weights). Every key pose of the
creature's stand, walk, attack and death (--all: every sequence on the base) is posed by the float64 port of the
client's skin (rigmath; checked against the client by rig_fetch --check), with the real weights and with ours.
Prints, per sequence: mean, 95th centile and largest vertex displacement between the two, in game units and as a
share of the creature's largest dimension; then how the weights compare (dominant bone equal, influences), and the
real bones whose vertices go most wrong (with the bone OURS gave them instead).
"""
import sys, json, pathlib, collections
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
import riglib as R, wlib
B, RM = R.B, R.RM
a = sys.argv; work = pathlib.Path(a[1]).resolve(); man = json.loads((work / 'manifest.json').read_text()); src = work / man['src']
def opt(n, d=None): return a[a.index(n) + 1] if n in a else d
sk = RM.Skeleton(B.decode_base((src / man['base_file']).read_bytes()))
real = B.decode_model((src / man['models'][0]).read_bytes()); ours = B.decode_model(pathlib.Path(a[2]).read_bytes())
assert real['vertices'] == ours['vertices'], 'not the same vertices'
V = np.array(real['vertices'], float); Wr = RM.weight_matrix(real['bone_weights'], sk.n); Wo = RM.weight_matrix(ours['bone_weights'], sk.n)
big = float((V.max(0) - V.min(0)).max()); nv = len(V)
seqs = man['sequences'] if '--all' in a else [s for s in man['sequences'] if s['seq'] in man['named'].values()]
name_of = {v: k for k, v in man['named'].items()}
rows = []; per_vertex = np.zeros(nv); frames = 0
for s in seqs:
    an = B.decode_anim((src / s['file']).read_bytes()); tb = RM.channel_table(an, sk, range(s['frames'] + 1)); t = RM.step_times(tb)
    K = sk.skin_matrices(tb[t]); d = np.linalg.norm(RM.skin_fast(V, Wr, K) - RM.skin_fast(V, Wo, K), axis=-1)          # (poses, nv)
    rows.append(dict(seq=s['seq'], name=name_of.get(s['seq'], ''), poses=len(t), mean=float(d.mean()), p95=float(np.quantile(d, .95)), max=float(d.max()),
                     moved_over_1pct=float((d > big * .01).mean()), moved_over_3pct=float((d > big * .03).mean())))
    per_vertex = np.maximum(per_vertex, d.max(0)); frames += len(t)
    r = rows[-1]; print(f"seq {r['seq']} {r['name']:6s} {r['poses']:3d} poses: mean {r['mean']:.2f}, 95th centile {r['p95']:.2f}, largest {r['max']:.1f} units"
                        f" ({100 * r['p95'] / big:.2f}% / {100 * r['max'] / big:.1f}% of {big:.0f}); vertices off by more than 1% of the size {100 * r['moved_over_1pct']:.1f}%, more than 3% {100 * r['moved_over_3pct']:.1f}%")
dr = Wr.argmax(1); do = Wo.argmax(1); ir = collections.Counter(len(w) for w in real['bone_weights']); io = collections.Counter(len(w) for w in ours['bone_weights'])
allm = float(np.mean([r['mean'] for r in rows])); print(f"OVERALL: {frames} poses; mean of sequence means {allm:.2f} units; worst vertex {per_vertex.max():.1f} units; vertices that are ever off by more than 1% of the size: {100 * (per_vertex > .01 * big).mean():.1f}%, 3%: {100 * (per_vertex > .03 * big).mean():.1f}%")
print(f"weights: dominant bone equal on {100 * (dr == do).mean():.1f}% of vertices; L1 distance of the weight rows, mean {np.abs(Wr - Wo).sum(1).mean() / 2:.3f}; influences real {dict(sorted(ir.items()))} ours {dict(sorted(io.items()))}; bones used real {len(set(np.nonzero(Wr.sum(0))[0]))} ours {len(set(np.nonzero(Wo.sum(0))[0]))}")
def cents(W):
    s = W.sum(0); ok = s > 0; return {int(b): (W[:, b] @ V) / s[b] for b in np.nonzero(ok)[0]}
cr, co = cents(Wr), cents(Wo); gaps = {b: float(np.linalg.norm(cr[b] - co[b])) for b in cr if b in co}
print(f"bone weighted-centroid gap (the gate's measure): median {np.median(list(gaps.values())):.1f} units over {len(gaps)} bones; real bones ours never uses: {sorted(set(cr) - set(co))}; worst five {[(b, round(g)) for b, g in sorted(gaps.items(), key=lambda kv: -kv[1])[:5]]}")
acc = collections.defaultdict(list)
for i in range(nv): acc[int(dr[i])].append(i)
worst = sorted(acc.items(), key=lambda kv: -per_vertex[kv[1]].mean() * len(kv[1]))[:int(opt('--worst', 8))]
for b, idx in worst:
    alt = collections.Counter(int(do[i]) for i in idx).most_common(3)
    print(f"  real bone {b:3d} (parent {sk.parent[b]}, depth {sk.depth[b]}): {len(idx)} vertices, worst displacement mean {per_vertex[idx].mean():.1f} max {per_vertex[idx].max():.1f}; ours gave them to {alt}")
if opt('--json'): pathlib.Path(opt('--json')).write_text(json.dumps(dict(rows=rows, overall_mean=allm, worst_vertex=float(per_vertex.max()), dominant_equal=float((dr == do).mean()), per_vertex=per_vertex.tolist()), indent=1))
