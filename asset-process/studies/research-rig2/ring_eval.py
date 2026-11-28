"""The ring rule against the real painting on MANY creatures, in motion (no drawing: the float64 port of the skin).
For each creature: the real weights are reduced to each vertex's heaviest bone; the rings are painted by rule in each
style; every version is posed through up to --max-seqs of the creature's sequences (a pose every 6 cycles) and the
mean vertex displacement from the REAL weights' pose is taken, as a share of the creature's largest dimension.
usage: ring_eval.py [--max-seqs 4] [--only NPC,..] [--json OUT]
Prints per creature: one-influence share | error rigid | soft | medium | best style; then the totals by softness class."""
import sys, json, pathlib, collections
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
import riglib as R, wlib
import ring_census as RC
L, B, RM = R.L, R.B, R.RM
a = sys.argv
def opt(n, d=None): return a[a.index(n) + 1] if n in a else d
todo = RC.creatures()
if opt('--only'): ids = {int(x) for x in opt('--only').split(',')}; todo = [x for x in todo if x['npc'] in ids]
MAXS = int(opt('--max-seqs', 4)); rows = []
def quant(W, n, steps=4, mx=2):
    Wq = np.zeros_like(W)
    for i, row in enumerate(W):
        idx = np.argsort(-row)[:mx]; idx = idx[row[idx] > 0]; w = row[idx] / row[idx].sum(); q = np.round(w * steps).astype(int); q[0] += steps - q.sum(); Wq[i, idx] = np.maximum(q, 0) / steps
    return wlib.dense(wlib.to_bytes(Wq, mx, 0.06, 0.0), n)
for x in todo:
    sk = RM.Skeleton(B.decode_base(L.base_bytes(x['base']))); ang, share, ns = RC.motion(sk, x['base'])
    seqs = sorted(L.seqs_on_base(x['base']), key=lambda s: -s[1])[:MAXS]; Ks = []
    for sid, n in seqs:
        try: aid, _ = L.anim_id(sid); an = B.decode_anim(L.anim_bytes(aid))
        except Exception: continue
        Ks.append(sk.skin_matrices(RM.channel_table(an, sk, range(0, n + 1, 6))))
    if not Ks: continue
    K = np.concatenate(Ks); err = collections.defaultdict(float); nv = 0; allV = []
    for mid in x['models']:
        m = B.decode_model(L.model_bytes(mid))
        if m['bone_weights'] is None: continue
        V = np.array(m['vertices'], float); F = np.array(m['faces']); Wr = RM.weight_matrix(m['bone_weights'], sk.n); dom = Wr.argmax(1); allV.append(V)
        owned = np.zeros(sk.n, bool); owned[np.unique(dom)] = True; kinds = wlib.joint_kinds(sk, owned, ang, share)
        P0 = RM.skin_fast(V, Wr, K)
        cand = dict(rigid=np.eye(sk.n)[dom], top2=wlib.dense(wlib.to_bytes(Wr, 2, 0, 0), sk.n))
        for st in ('soft', 'medium'): cand[st] = quant(wlib.paint_rings(sk, V, F, dom, kinds, style=st), sk.n)
        cand['soft-tree'] = quant(wlib.paint_rings(sk, V, F, dom, None, style='soft'), sk.n)
        for k, W in cand.items(): err[k] += float(np.linalg.norm(P0 - RM.skin_fast(V, W, K), axis=-1).mean(0).sum())
        nv += len(V)
    allV = np.concatenate(allV); big = float((allV.max(0) - allV.min(0)).max())
    r = dict(npc=x['npc'], name=x['name'], one=x['one'], size=big, **{k: 100 * v / nv / big for k, v in err.items()}); rows.append(r)
    best = min(('rigid', 'soft', 'medium'), key=lambda k: r[k])
    print(f"{x['npc']:6d} {x['name'][:22]:22s} one {x['one']:.2f} | error as % of size: rigid {r['rigid']:.3f} | soft {r['soft']:.3f} | medium {r['medium']:.3f} | soft, kinds from the tree only {r['soft-tree']:.3f} | real two heaviest {r['top2']:.3f} | best {best}", flush=True)
print('\nBY SOFTNESS CLASS (share of one-influence vertices: soft under 0.4, medium 0.4 to 0.65, hard over 0.65): creatures | mean error % of size rigid / soft / medium | creatures where each style beats rigid')
for name, lo, hi in (('soft', 0, .4), ('medium', .4, .65), ('hard', .65, 1.01), ('all', 0, 1.01)):
    r = [x for x in rows if lo <= x['one'] < hi]
    if r: print(f"  {name:6s} {len(r):2d} | {np.mean([x['rigid'] for x in r]):.3f} / {np.mean([x['soft'] for x in r]):.3f} / {np.mean([x['medium'] for x in r]):.3f} | soft beats rigid on {sum(x['soft'] < x['rigid'] for x in r)}, medium beats rigid on {sum(x['medium'] < x['rigid'] for x in r)}; real two heaviest {np.mean([x['top2'] for x in r]):.3f}")
if opt('--json'): pathlib.Path(opt('--json')).write_text(json.dumps(rows, indent=1))
