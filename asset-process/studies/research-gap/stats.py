"""The test of PREREG.md: direction, leave-one-out with balanced errors, the BEST condition, and chance by relabelling.

    P stats.py [--perm N]   -> data/stats.json, data/table.md
"""
import sys, pathlib, json
import numpy as np
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from measures import KEYS, DIRECTION

def best_cut(x, y):
    """x values, y True = REAL; REAL predicted on the side given by sign already applied (higher = REAL).
    Midpoints between sorted distinct values; fewest balanced errors; ties -> middle of the tied cuts."""
    u = np.unique(x)
    if len(u) < 2: return None
    cuts = (u[:-1] + u[1:]) / 2
    nR, nO = y.sum(), (~y).sum()
    # errors: REAL at or below cut, OURS above cut
    xr = np.sort(x[y]); xo = np.sort(x[~y])
    eR = np.searchsorted(xr, cuts, side='right') / max(nR, 1)
    eO = (nO - np.searchsorted(xo, cuts, side='right')) / max(nO, 1)
    err = eR + eO; m = err.min(); idx = np.nonzero(np.isclose(err, m))[0]
    if len(idx) % 2: return cuts[idx[len(idx) // 2]]
    return (cuts[idx[len(idx) // 2 - 1]] + cuts[idx[len(idx) // 2]]) / 2

def loo(x, y):
    """returns predicted REAL (bool) for each left-out item"""
    n = len(x); pred = np.zeros(n, bool)
    for i in range(n):
        k = np.arange(n) != i
        c = best_cut(x[k], y[k])
        pred[i] = (x[i] > c) if c is not None else True
    return pred

def evaluate(x, y, best):
    pred = loo(x, y)
    accR = (pred & y).sum() / y.sum(); accO = (~pred & ~y).sum() / (~y).sum()
    bal = (accR + accO) / 2
    best_called = int((~pred & best).sum())
    return bal, best_called, pred

def main():
    nperm = int(sys.argv[sys.argv.index('--perm') + 1]) if '--perm' in sys.argv else 20000
    d = json.loads((HERE / 'data' / 'measures.json').read_text())
    names = list(d)
    res = {}; rng = np.random.default_rng(1)
    for k in KEYS:
        nm = [n for n in names if d[n]['m'].get(k) is not None]
        x = np.array([d[n]['m'][k] for n in nm], float)
        s = 1.0 if DIRECTION[k] == 'higher' else -1.0
        y = np.array([d[n]['group'] == 'REAL' for n in nm]); best = np.array([d[n]['group'] == 'BEST' for n in nm])
        bal, bc, pred = evaluate(s * x, y, best)
        # raw direction check: medians
        medR, medO = float(np.median(x[y])), float(np.median(x[~y]))
        dir_ok = (medR > medO) if s > 0 else (medR < medO)
        survive = bool(dir_ok and bal >= 0.80 and bc >= 4)
        # chance: relabel, same counts
        nO, nB = int((~y).sum()), int(best.sum()); hits = 0
        for _ in range(nperm):
            perm = rng.permutation(len(nm)); o = perm[:nO]; b = perm[:nB]
            yy = np.ones(len(nm), bool); yy[o] = False; bb = np.zeros(len(nm), bool); bb[b] = True
            mr, mo = np.median(x[yy]), np.median(x[~yy])
            if not ((mr > mo) if s > 0 else (mr < mo)): continue
            b2, c2, _ = evaluate(s * x, yy, bb)
            hits += (b2 >= 0.80 and c2 >= 4)
        xr = x[y]
        res[k] = dict(direction=DIRECTION[k], n_real=int(y.sum()), n_ours=int((~y).sum()), n_best=nB, real_median=medR, ours_median=medO,
                      dir_ok=bool(dir_ok), loo_balanced=float(bal), best_called_ours=bc, survives=survive, p_chance=hits / nperm,
                      real_p10=float(np.quantile(xr, .1)), real_p90=float(np.quantile(xr, .9)), real_min=float(xr.min()), real_max=float(xr.max()),
                      cut=float(s * best_cut(s * x, y)),
                      values={n: float(v) for n, v in zip(nm, x)}, loo_called_real={n: bool(p) for n, p in zip(nm, pred)})
        print(f"{k:14s} dir {'ok ' if dir_ok else 'NO '} LOO bal {bal:.2f} best-as-ours {bc}/{nB}  survive {survive}  p {hits / nperm:.4f}", flush=True)
    exp = sum(r['p_chance'] for r in res.values())
    res['_chance_expected_survivors'] = exp
    (HERE / 'data' / 'stats.json').write_text(json.dumps(res, indent=1))
    print('expected survivors by chance:', round(exp, 3))

if __name__ == '__main__':
    main()
