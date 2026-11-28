"""bands.py: the definition measures by part of the standing figure (front yaw 192 and back yaw 1216, zoom 300, mean of
the two): trunk and arms (heights 114..164) against legs and feet (0..114). Tests whether the redesigned set lost its
definition in the body piece (which the owner preferred alone) or below it. Writes bands.json."""
import sys, pathlib, json
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
import lib as L, deflib as D

names = [r[0] for r in L.REFS] + ['earlier', 'redesigned']
entries = [L.ref_outfit(r, helm=False) for r in L.REFS] + [L.SET_V2, L.SET_V3] + ['none']
out = {n: {} for n in names}
for yaw in (192, 1216):
    a = L.pose(L.ROOT / 'render/research-fit-definition' / f'bands-{yaw}.png', entries, '--zoom', 300, '--yaw', yaw, '--cell', '260x420')
    cw = a.shape[1] // len(entries); bg = a[2, 2]
    none = a[:, -cw:]; ys = np.nonzero((np.abs(none - bg).sum(2) > 0).any(1))[0]; ytop, ybot = ys.min(), ys.max()
    row = lambda h: int(round(ytop + (196 - h) * (ybot - ytop) / 196.0))
    for k, n in enumerate(names):
        c = a[:, k * cw:(k + 1) * cw]
        for band, (h0, h1) in (('trunk', (114, 164)), ('legs', (-5, 114))):
            cc = c.copy(); cc[:row(h1)] = bg; cc[row(h0):] = bg
            d = D.analyse(cc, bg)
            for key in ('step', 'step_level', 'lines', 'line_dark', 'regions', 'reg_big', 'p05', 'dark'):
                out[n].setdefault(band, {}).setdefault(key, []).append(d[key])
res = {n: {b: {k: round(float(np.mean(v)), 2) for k, v in d.items()} for b, d in bd.items()} for n, bd in out.items()}
json.dump(res, open(pathlib.Path(__file__).parent / 'bands.json', 'w'), indent=1)
for band in ('trunk', 'legs'):
    print(band); print(f"{'set':12}" + ''.join(f'{k:>11}' for k in ('step', 'step_level', 'lines', 'line_dark', 'regions', 'reg_big', 'p05', 'dark')))
    for n in names:
        print(f'{n:12}' + ''.join(f'{res[n][band][k]:>11}' for k in ('step', 'step_level', 'lines', 'line_dark', 'regions', 'reg_big', 'p05', 'dark')))
