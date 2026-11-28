"""Timing of a keyframe (skeletal) sequence, measured: what an animation author studies before posing.

usage: rig_timing.py WORKDIR SEQ[,SEQ..] | --file ANIM.dat [--model PART.bin] [--base BASE.dat]   [--bones N]
For each sequence: length in cycles and seconds; key spacing; distinct poses; then the SPEED CURVE: at each key pose
the mean and the largest vertex travel since the previous pose, in units per cycle (a bar per pose); where the speed
peaks (the strike), the longest near-still stretch (a hold), how far the whole body's root travels; the N bones
that turn most (range of each bone's rotation in degrees, in its parent's frame) and the 5 vertices that travel most.
"""
import sys, json, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
import riglib as R
B, RM = R.B, R.RM
a = sys.argv; work = pathlib.Path(a[1]).resolve(); man = json.loads((work / 'manifest.json').read_text()); src = work / man['src']
def opt(n, d=None): return a[a.index(n) + 1] if n in a else d
sk = RM.Skeleton(B.decode_base(pathlib.Path(opt('--base') or (src / man['base_file'])).read_bytes()))
m = B.decode_model(pathlib.Path(opt('--model') or (src / man['models'][0])).read_bytes()); V = np.array(m['vertices'], float); W = RM.weight_matrix(m['bone_weights'], sk.n)
big = float((V.max(0) - V.min(0)).max()); nb = int(opt('--bones', 8))
jobs = [(opt('--file'), B.decode_anim(pathlib.Path(opt('--file')).read_bytes()), None)] if opt('--file') else \
    [(s['seq'], B.decode_anim((src / s['file']).read_bytes()), s['frames']) for q in a[2].split(',') for s in man['sequences'] if s['seq'] == int(q)]
def measure(name, an, n):
    n = n or an['range'][1]; tb = RM.channel_table(an, sk, range(n + 1)); t = RM.step_times(tb); K = sk.skin_matrices(tb[t]); P = RM.skin_fast(V, W, K)
    gaps = np.diff(t); d = np.linalg.norm(np.diff(P, axis=0), axis=-1)                   # (poses - 1, nv) travel between poses
    mean = d.mean(1) / gaps; mx = d.max(1) / gaps
    print(f"\n{name}: {n} cycles = {n * 0.02:.2f} s; {len(t)} distinct poses; key spacing (cycles: count) {dict(zip(*[x.tolist() for x in np.unique(gaps, return_counts=True)]))}; size {big:.0f}")
    top = mean.max() or 1
    for k in range(len(mean)):
        print(f"  cycle {t[k]:3d}-{t[k + 1]:3d}  mean {mean[k]:6.2f}  max {mx[k]:6.2f} units/cycle  {'#' * int(round(40 * mean[k] / top))}")
    peak = int(mean.argmax()); still = mean < 0.15 * top; best = (0, 0); run = 0
    for k, s in enumerate(still):
        run = run + gaps[k] if s else 0
        if run > best[0]: best = (int(run), int(t[k + 1]))
    root = sk.parent.index(-1); rt = tb[t][:, root, 3:6]
    print(f"  fastest between cycles {t[peak]} and {t[peak + 1]} ({100 * t[peak] / n:.0f}% of the way): mean {mean[peak]:.1f}, max {mx[peak]:.1f} units/cycle; mean speed over the whole {mean.mean():.2f};"
          f" peak / mean {mean[peak] / max(mean.mean(), 1e-9):.1f}; longest near-still stretch {best[0]} cycles ending at {best[1]}; root travels {np.ptp(rt, axis=0).round(1).tolist()} units; first and last pose differ by {np.linalg.norm(P[0] - P[-1], axis=-1).max():.1f} units at most")
    rng = np.degrees(np.ptp(RM.unwrap(tb[t][..., 0:3]), axis=0)).max(-1); order = np.argsort(-rng)[:nb]
    print('  bones that turn most (degrees of range): ' + ', '.join(f'{int(b)}: {rng[b]:.0f}' for b in order) + f'; bones turning over 5 degrees: {int((rng > 5).sum())} of {sk.n}')
    trav = np.linalg.norm(P - P[0], axis=-1).max(0); print(f'  farthest any vertex gets from its first pose: {trav.max():.0f} units ({100 * trav.max() / big:.0f}% of the size); median vertex {np.median(trav):.0f}')
    return dict(name=str(name), cycles=n, poses=len(t), mean=mean.tolist(), times=[int(x) for x in t])
out = [measure(*j) for j in jobs]
if opt('--json'): pathlib.Path(opt('--json')).write_text(json.dumps(out))
