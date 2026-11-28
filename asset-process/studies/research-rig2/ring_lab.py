"""The ring rule on a REAL creature: the real weights are reduced to ONE bone per vertex (or a one-bone assignment
is read from other model files), the rings are painted by rule (wlib.paint_rings), and the result is scored against
the real weights in motion (the float64 port of the client's skin; every key pose of stand, walk, attack, death).

usage: ring_lab.py WORKDIR [--dom real | --dom-from DIR] [--out DIR] [--kinds motion|tree|none] [--style soft|medium|hard] [--amp A --reach R --hops N]
                   [--steps K] [--max N] [--floor F] [--min-angle A] [--compare]
  --dom real       each vertex's heaviest REAL bone (the assignment an oracle would make; isolates the ring rule)
  --dom-from DIR   the heaviest bone in DIR/part-K.bin (weights assigned by rule: nothing of the real weights is read)
  --kinds motion   joint kinds from the creature's own sequences (rigid under --min-angle degrees, hinge when 90% of
                   the turning is about one axis), chain from the tree;  tree: from the bone tree alone;  none: no rings
  --steps K        shares rounded to 1/K (default 4: 128+127, 191+64);  0 = unrounded bytes
  --out DIR        write part-K.bin with the new weights (for rig_gate.py)
  --compare        also print the reference rows: one bone a vertex, and the real weights' two heaviest
"""
import sys, json, pathlib, collections
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
import riglib as R, wlib
from ring_census import rotvec
B, RM = R.B, R.RM
a = sys.argv; work = pathlib.Path(a[1]).resolve(); man = json.loads((work / 'manifest.json').read_text()); src = work / man['src']
def opt(n, d=None): return a[a.index(n) + 1] if n in a else d
sk = RM.Skeleton(B.decode_base((src / man['base_file']).read_bytes()))
models = [B.decode_model((src / f).read_bytes()) for f in man['models']]
UNW = [m['bone_weights'] is None for m in models]             # a model with no weights at all does not move: passed through
for m in models:
    if m['bone_weights'] is None: m['bone_weights'] = [[] for _ in m['vertices']]
named = [s for s in man['sequences'] if s['seq'] in man['named'].values()]
tables = {}
for s in man['sequences']:
    an = B.decode_anim((src / s['file']).read_bytes()); tb = RM.channel_table(an, sk, range(s['frames'] + 1)); tables[s['seq']] = tb[RM.step_times(tb)]

def motion():
    rest = sk.local[:, :3, :3]; rest = rest / np.linalg.norm(rest, axis=-1, keepdims=True); rv = []
    for tb in tables.values():
        M = RM.compose(tb[..., 0:3], tb[..., 3:6], np.ones_like(tb[..., 6:9]))[..., :3, :3]; rv.append(rotvec(np.swapaxes(rest, -1, -2)[None] @ M))
    rv = np.concatenate(rv); ang = np.degrees(np.linalg.norm(rv, axis=-1)).max(0); share = np.zeros(sk.n)
    for i in range(sk.n):
        x = rv[:, i] - rv[:, i].mean(0); s = np.linalg.svd(x, compute_uv=False) ** 2; share[i] = s[0] / s.sum() if s.sum() > 1e-12 else 1.0
    rng = np.degrees(np.array([np.linalg.norm(rv[:, i].max(0) - rv[:, i].min(0)) for i in range(sk.n)]))
    return np.maximum(ang, rng), share

Ks = {s['seq']: sk.skin_matrices(tables[s['seq']]) for s in named}
def score(Wnew, label):
    """Wnew: list of dense matrices per model. Mean vertex displacement from the real weights, per named sequence."""
    means = []; over3 = []; nvt = 0
    per = [np.zeros(len(m['vertices'])) for m in models]
    for s in named:
        K = Ks[s['seq']]; tot = 0; cnt = 0
        for k, m in enumerate(models):
            V = np.array(m['vertices'], float); Wr = RM.weight_matrix(m['bone_weights'], sk.n)
            d = np.linalg.norm(RM.skin_fast(V, Wr, K) - RM.skin_fast(V, Wnew[k], K), axis=-1); tot += d.sum(); cnt += d.size; per[k] = np.maximum(per[k], d.max(0))
        means.append(tot / cnt)
    allV = np.concatenate([np.array(m['vertices'], float) for m in models]); big = float((allV.max(0) - allV.min(0)).max()); pv = np.concatenate(per)
    inf = collections.Counter(int((w > 0).sum()) for Wk in Wnew for w in Wk)
    print(f"{label:46s} mean vertex error {np.mean(means):6.2f} units (per sequence {', '.join(f'{x:.2f}' for x in means)}); ever off by over 1% of the size {100 * (pv > .01 * big).mean():5.1f}%, over 3%: {100 * (pv > .03 * big).mean():5.1f}%; influences {dict(sorted(inf.items()))}")
    return float(np.mean(means))

def quant(W, steps, mx, floor):
    """The game's form, as rig_weights.py does it: the mx heaviest, shares in 1/steps, bytes summing to 255."""
    if steps:
        Wq = np.zeros_like(W)
        for i, row in enumerate(W):
            idx = np.argsort(-row)[:mx]; idx = idx[row[idx] > 0]; w = row[idx] / row[idx].sum(); q = np.round(w * steps).astype(int); q[0] += steps - q.sum(); Wq[i, idx] = np.maximum(q, 0) / steps
        W = Wq
    by = wlib.to_bytes(W, mx, floor, 0.0); return by, wlib.dense(by, sk.n)

real = [RM.weight_matrix(m['bone_weights'], sk.n) for m in models]
if opt('--score'):                                   # just score the weights of DIR/part-K.bin (any number of model files) and stop
    d = pathlib.Path(opt('--score')); ours = [B.decode_model((d / f'part-{k}.bin').read_bytes()) for k in range(len(models))]
    for o in ours:
        if o['bone_weights'] is None: o['bone_weights'] = [[] for _ in o['vertices']]
    assert all(o['vertices'] == m['vertices'] for o, m in zip(ours, models)), 'not the same vertices'
    score([RM.weight_matrix(o['bone_weights'], sk.n) for o in ours], f'weights of {d}'); sys.exit(0)
if opt('--dom-from'):
    d = pathlib.Path(opt('--dom-from')); doms = [RM.weight_matrix(B.decode_model((d / f'part-{k}.bin').read_bytes())['bone_weights'], sk.n).argmax(1) for k in range(len(models))]; src_label = f'rule-assigned bones ({d.parent.name})'
else:
    doms = [w.argmax(1) for w in real]; src_label = 'real heaviest bone'
if '--compare' in a:
    score(real, 'real weights (ceiling)')
    score([wlib.dense(wlib.to_bytes(w, 2, 0, 0), sk.n) for w in real], 'real, two heaviest kept')
    score([np.eye(sk.n)[w.argmax(1)] * (w.sum(1, keepdims=True) > 0) for w in real], 'real, ONE heaviest kept (rigid)')
onehot = [np.eye(sk.n)[d] * (w.sum(1, keepdims=True) > 0) for d, w in zip(doms, real)]
if opt('--dom-from'): score(onehot, f'{src_label}, one bone a vertex')
mode = opt('--kinds', 'motion'); steps = int(opt('--steps', 4)); mx = int(opt('--max', 2)); out_bytes = []; newW = []; rep_all = []
ang, share = motion()
for k, m in enumerate(models):
    V = np.array(m['vertices'], float); F = np.array(m['faces']); owned = np.zeros(sk.n, bool); owned[np.unique(doms[k])] = True
    if UNW[k]: out_bytes.append(None); newW.append(real[k]); continue
    if mode == 'real2': W = wlib.dense(wlib.to_bytes(real[k], 2, 0, 0), sk.n); steps = 0
    elif mode == 'real': W = real[k]; steps = 0; mx = 9
    elif mode == 'none': W = onehot[k]
    else:
        kinds = wlib.joint_kinds(sk, owned, ang if mode == 'motion' else None, share if mode == 'motion' else None, min_angle=float(opt('--min-angle', 8)))
        rep = []; W = wlib.paint_rings(sk, V, F, doms[k], kinds, style=opt('--style', 'soft'), hops=int(opt('--hops')) if opt('--hops') else None, amp=float(opt('--amp')) if opt('--amp') else None, reach=float(opt('--reach')) if opt('--reach') else None, report=rep, webs=(opt('--webs', 'on') if opt('--webs', 'on') == 'distance' else opt('--webs', 'on') == 'on'), web_min=float(opt('--web-min', 0.45)), web_share=float(opt('--web-share', 0.25)), len_cap=float(opt('--len-cap', wlib.RING_LEN_CAP)),
                                      lateral=float(opt('--lateral', wlib.RING_LATERAL)), centre=opt('--centre', 'joint'), min_boundary=int(opt('--min-boundary', 3))); rep_all += rep
    by, Wd = quant(W, steps, mx, float(opt('--floor', 0.06))); out_bytes.append(by); newW.append(Wd)
nweb = sum(j.get('webs', 0) for j in rep_all); rep_all = [j for j in rep_all if 'kind' in j]; kc = collections.Counter(j['kind'] for j in rep_all)
print(f"joints painted: {len(rep_all)} {dict(kc)}; vertices given a cross share {sum(j['painted'] for j in rep_all)}; web vertices {nweb}")
score(newW, f'{src_label} + rings by rule ({mode}, style {opt("--style", "soft")}, steps {steps}, max {mx})')
if opt('--out'):
    o = pathlib.Path(opt('--out')); o.mkdir(parents=True, exist_ok=True)
    for k, m in enumerate(models):
        m2 = dict(m); m2['bone_weights'] = out_bytes[k]; (o / f'part-{k}.bin').write_bytes(B.encode_model(m2, hints=False))
    assert all(sum(x for _, x in w) == 255 for by in out_bytes if by is not None for w in by)
    print('wrote', o)
