"""gallery.py: the study set: real effects grouped by JOB, drawn by id through the harness at game size, as contact sheets
and moving pictures, and measured ON SCREEN (how many pixels the effect visibly changes (a channel by 8 or more), how far those pixels are from the scene
behind them, over its life).
usage: gallery.py sheets | gifs | measure | all      -> render/research-effects-gallery/, data/gallery-measure.json
Names are the cache's own (gameval archive 24, group 8). Jobs are assigned from the name and from what the server copy
uses the id for (plugins/bosses/data/*.json), and checked by drawing.
"""
import sys, pathlib, json
sys.dont_write_bytecode = True; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import fx
from fxlib import *
import numpy as np
from PIL import Image

OUT = RENDER / 'research-effects-gallery'
VISIBLE = 8   # a pixel counts as changed by the effect when a channel moves by at least this much (0..255): a face at alpha 253 does not count
# job -> (harness job, [ids])
PICKS = {
    'projectile': ('proj', [10, 1997, 1999, 2004, 2007, 1764, 1766, 1340, 1580, 1583, 1593, 2206, 3445, 3254, 1477, 1711]),
    'impact': ('actor', [1998, 2008, 2005, 1765, 1582, 1584, 2205, 3255, 2712, 1478]),
    'cast-charge-aura': ('actor', [2016, 2015, 2017, 1760, 2125, 3243, 2141, 1558, 1597, 3280]),
    'ground-hazard': ('spot', [1654, 1767, 2013, 3857, 3429, 1356, 2432, 2453]),
    'telegraph': ('spot', [1446, 2198, 2199, 3415, 2667, 1595, 2146, 2440]),
    'hit-after-telegraph': ('spot', [2197, 3370, 1445, 2216, 3262, 2148]),
}
HEIGHT = {'impact': 100, 'cast-charge-aura': 0}


def screen_stats(effect, job, cache=fx.SRC, height=0, n=12, proj=(172, 124, 16, 64, 60), size=(512, 334)):
    """Per moment: pixels the effect changes against the same scene without it, their box, mean colour, and mean distance
    (0..255, per channel average) from what is behind. Game-size lens, the game's 512 x 334 view."""
    total = fx.life(effect, job, cache, proj)
    cyc = fx.moments(n, total)
    A, _ = fx.render(effect, job, cyc, cache, False, size, height, proj)
    # the same scene without the effect: start the effect far in the future
    import os
    tag = f'{OUT.as_posix()}/_e{os.getpid()}'
    L = fx.scene(job, False, size) + [f'render {",".join(str(4 + c) for c in cyc)} {tag}']
    fx.run(L, cache)
    E = fx.frames(tag, [4 + c for c in cyc])
    rows = []
    for c, a, e in zip(cyc, A, E):
        a = np.asarray(a).astype(int); e = np.asarray(e).astype(int)
        m = (np.abs(a - e) >= VISIBLE).any(axis=2)
        k = int(m.sum())
        if k == 0: rows.append(dict(c=c, px=0)); continue
        ys, xs = np.nonzero(m)
        col = a[m].mean(axis=0); dist = np.abs(a[m] - e[m]).mean()
        mx = a[m].max(axis=1); mn = a[m].min(axis=1)
        rows.append(dict(c=c, px=k, box=[int(xs.max() - xs.min() + 1), int(ys.max() - ys.min() + 1)], rgb=[int(v) for v in col], dist=round(float(dist), 1),
                         sat=round(float(((mx - mn) / np.maximum(mx, 1)).mean()), 2), bright=round(float(mx.mean()), 1)))
    return dict(total=total, moments=rows)


def main(what):
    OUT.mkdir(parents=True, exist_ok=True)
    nm = names(8)
    if what in ('sheets', 'all'):
        for jobname, (job, ids) in PICKS.items():
            fx.sheet(OUT / f'sheet-{jobname}.png', [f'id:{i}' for i in ids], job, n=6, height=HEIGHT.get(jobname, 0), size=(256, 200))
            print('sheet', jobname, len(ids))
    if what in ('gifs', 'all'):
        for jobname, (job, ids) in PICKS.items():
            fx.gif(OUT / f'motion-{jobname}.gif', [f'id:{i}' for i in ids], job, height=HEIGHT.get(jobname, 0), size=(224, 180), step=2, cols=4)
            print('gif', jobname)
    if what in ('measure', 'all'):
        res = {}
        for jobname, (job, ids) in PICKS.items():
            for i in ids:
                s = screen_stats(f'id:{i}', job, height=HEIGHT.get(jobname, 0))
                res[i] = dict(job=jobname, name=nm.get(i, ''), **s)
                px = [m['px'] for m in s['moments']]
                d = [m['dist'] for m in s['moments'] if m['px']]
                print(f'{jobname:22s} {i:5d} {nm.get(i, "")[:44]:44s} cycles {s["total"]:4d} px max {max(px):6d} mean {int(np.mean(px)):6d} first {px[0]:5d} last {px[-1]:5d} dist {np.mean(d) if d else 0:5.1f}')
        jdump(HERE / 'data' / 'gallery-measure.json', res)
    p = OUT / '_e'


if __name__ == '__main__':
    main(sys.argv[1] if len(sys.argv) > 1 else 'all')
