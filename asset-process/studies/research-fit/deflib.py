"""deflib.py: how a drawn figure is divided into readable parts ("definition"), as numbers from the game's own picture.

Input: one cell of a pose.py picture (exact palette colours, no anti-aliasing) and the background colour.
grey = (r + g + b) / 3, 0..255. The figure = pixels that differ from the background. T = 24 grey levels unless said.

  area         figure pixels
  step         tone steps: neighbouring figure pixels (right and down) whose grey differs by T or more, per 100 figure
               pixels ("how much line there is in the figure"). step16 / step40: the same at T = 16 and 40.
  step_level, step_upright   the steps split by direction: level lines (bands across the trunk and limbs) and
               upright lines (running along them), each per 100 figure pixels; they add up to step
  step_len     the same count divided by the square root of the area (a length in figure-sizes; does not grow with zoom)
  step_mat     share of those steps where the two pixels are different MATERIALS (hue and saturation differ: a trim,
               a cloth, skin against metal)
  step_dark    share where the material is the same (or one side is too dark to tell) and the darker side is a dark
               (lit lightness <= 22 of 127): a dark line or recess
  step_tone    the rest: one material, both sides lit: a change of plane or a painted tone
  line_dark    pixels that are a dark LINE: darker by T than the pixels 2 or 3 away on BOTH sides (up/down or left/right),
               per 100 figure pixels
  line_light   the same, lighter than both sides: a light trim or a lit edge
  line_mat     pixels whose material differs from both sides while the two sides agree: a trim of a second material
  lines        line_dark + line_light + line_mat, counted once
  regions      readable regions: connected patches of 12 or more pixels within which no neighbour step reaches 12 grey
               levels; reg_big = share of the figure in its largest such patch (1 = one undivided mass);
               reg_cover = share of the figure inside readable regions
  p05, p50, p95  grey percentiles; range = p95 - p05; dark = share of figure pixels with grey < 30
  contrast     median grey difference across the steps
"""
import numpy as np
from scipy import ndimage
import lib as L

T = 24


def same_mat(h1, s1, l1, h2, s2, l2):
    dh = np.abs(h1 - h2); dh = np.minimum(dh, 64 - dh)
    grey = (s1 == 0) & (s2 == 0)
    return grey | ((dh <= 2) & (np.abs(s1 - s2) <= 1))


def analyse(cell, bg):
    c = cell.astype(np.int32)
    m = np.abs(c - bg).sum(2) > 0
    area = int(m.sum())
    if area < 50:
        return None
    g = c.sum(2) / 3.0
    hh, ss, ll, ok = L.hsl_of(c.reshape(-1, 3))
    hh = hh.reshape(m.shape); ss = ss.reshape(m.shape); ll = ll.reshape(m.shape); ok = ok.reshape(m.shape)
    out = dict(area=area)
    pairs = []
    for ax in (0, 1):
        a = [slice(None), slice(None)]; b = [slice(None), slice(None)]
        a[ax] = slice(0, -1); b[ax] = slice(1, None); a = tuple(a); b = tuple(b)
        both = m[a] & m[b]
        d = np.abs(g[a] - g[b])
        pairs.append((a, b, both, d))
    for t, name in ((16, 'step16'), (T, 'step'), (40, 'step40')):
        out[name] = round(100.0 * sum(int((both & (d >= t)).sum()) for a, b, both, d in pairs) / area, 2)
    # level lines (steps between a pixel and the one below it: bands ACROSS the trunk and limbs of a standing figure)
    # against upright lines (steps between a pixel and the one beside it: lines running ALONG them)
    out['step_level'] = round(100.0 * int((pairs[0][2] & (pairs[0][3] >= T)).sum()) / area, 2)
    out['step_upright'] = round(100.0 * int((pairs[1][2] & (pairs[1][3] >= T)).sum()) / area, 2)
    n = mat = dark = 0; diffs = []
    for a, b, both, d in pairs:
        e = both & (d >= T)
        n += int(e.sum())
        known = ok[a] & ok[b] & (ll[a] >= 10) & (ll[b] >= 10)
        sm = same_mat(hh[a], ss[a], ll[a], hh[b], ss[b], ll[b])
        ismat = e & known & ~sm
        isdark = e & ~ismat & (np.minimum(ll[a], ll[b]) <= 22)
        mat += int(ismat.sum()); dark += int(isdark.sum())
        diffs.append(d[e])
    diffs = np.concatenate(diffs) if diffs else np.array([0.0])
    out['step_len'] = round(n / area ** 0.5, 2)
    out['step_mat'] = round(mat / max(n, 1), 2); out['step_dark'] = round(dark / max(n, 1), 2)
    out['step_tone'] = round(1 - (mat + dark) / max(n, 1), 2)
    out['contrast'] = round(float(np.median(diffs)), 1) if len(diffs) else 0.0
    # lines
    ld = np.zeros_like(m); lg = np.zeros_like(m); lm = np.zeros_like(m)
    H, W = m.shape
    for ax in (0, 1):
        for k in (2, 3):
            def sh(arr, s):
                return np.roll(arr, s, axis=ax)
            valid = m & sh(m, k) & sh(m, -k)
            if ax == 0: valid[:k] = False; valid[-k:] = False
            else: valid[:, :k] = False; valid[:, -k:] = False
            g1, g2 = sh(g, k), sh(g, -k)
            ld |= valid & (g + T <= np.minimum(g1, g2))
            lg |= valid & (g - T >= np.maximum(g1, g2))
            known = ok & sh(ok, k) & sh(ok, -k) & (ll >= 10) & (sh(ll, k) >= 10) & (sh(ll, -k) >= 10)
            d1 = ~same_mat(hh, ss, ll, sh(hh, k), sh(ss, k), sh(ll, k))
            d2 = ~same_mat(hh, ss, ll, sh(hh, -k), sh(ss, -k), sh(ll, -k))
            s12 = same_mat(sh(hh, k), sh(ss, k), sh(ll, k), sh(hh, -k), sh(ss, -k), sh(ll, -k))
            lm |= valid & known & d1 & d2 & s12
    out['line_dark'] = round(100.0 * ld.sum() / area, 2); out['line_light'] = round(100.0 * lg.sum() / area, 2)
    out['line_mat'] = round(100.0 * lm.sum() / area, 2); out['lines'] = round(100.0 * (ld | lg | lm).sum() / area, 2)
    # readable regions: remove every pixel that takes part in a step of 12 or more, label what is left
    soft = m.copy()
    for a, b, both, d in pairs:
        e = both & (d >= 12)
        za = np.zeros_like(m); za[a] = e; zb = np.zeros_like(m); zb[b] = e
        soft &= ~za; soft &= ~zb
    lab, nlab = ndimage.label(soft)
    sizes = np.bincount(lab.ravel())[1:] if nlab else np.array([0])
    big = sizes[sizes >= 12]
    out['regions'] = int(len(big)); out['reg_big'] = round(float(sizes.max() / area), 2) if len(sizes) else 0.0
    out['reg_cover'] = round(float(big.sum() / area), 2)
    out['reg_per1000'] = round(1000.0 * len(big) / area, 1)
    gv = g[m]
    for q in (5, 50, 95):
        out[f'p{q:02d}'] = round(float(np.percentile(gv, q)), 1)
    out['range'] = round(out['p95'] - out['p05'], 1); out['dark'] = round(float((gv < 30).mean()), 3)
    out['mean'] = round(float(gv.mean()), 1)
    return out


def edge_picture(cell, bg):
    """a picture of what the measures see: steps white, dark lines blue, light lines yellow, material lines magenta"""
    c = cell.astype(np.int32); m = np.abs(c - bg).sum(2) > 0; g = c.sum(2) / 3.0
    img = np.zeros(c.shape, np.uint8); img[m] = (60, 60, 60)
    e = np.zeros_like(m)
    e[:-1] |= m[:-1] & m[1:] & (np.abs(g[:-1] - g[1:]) >= T)
    e[:, :-1] |= m[:, :-1] & m[:, 1:] & (np.abs(g[:, :-1] - g[:, 1:]) >= T)
    img[e] = (255, 255, 255)
    return img


KEYS = ['area', 'step16', 'step', 'step40', 'step_level', 'step_upright', 'step_len', 'step_mat', 'step_dark', 'step_tone', 'contrast', 'line_dark', 'line_light',
        'line_mat', 'lines', 'regions', 'reg_per1000', 'reg_big', 'reg_cover', 'p05', 'p50', 'p95', 'range', 'dark', 'mean']


def run_entries(png, entries, zoom=300, cell=(260, 420), yaws=(192, 1216), seqs=((819, 'walk'), (824, 'run'))):
    """draw each entry standing and through 8 frames of walk and run, front and back; return
    {entry_index: {'stand-front': {...}, 'walk-front': [8 dicts], ...}} and the cells for contact sheets"""
    import pathlib
    res = {k: {} for k in range(len(entries))}; cells = {k: {} for k in range(len(entries))}
    frames = ['bind'] + [str(i) for i in range(8)]
    for seq, sname in seqs:
        for yaw in yaws:
            vname = 'front' if yaw < 512 or yaw > 1536 else 'back'
            p = pathlib.Path(png); p = p.with_name(f'{p.stem}-{sname}-{vname}.png')
            a = L.pose(p, entries, '--seq', seq, '--frames', ','.join(frames), '--zoom', zoom, '--yaw', yaw, '--cell', f'{cell[0]}x{cell[1]}')
            ch = a.shape[0] // len(frames); cw = a.shape[1] // len(entries); bg = a[2, 2]
            for k in range(len(entries)):
                for fi, fr in enumerate(frames):
                    c = a[fi * ch:(fi + 1) * ch, k * cw:(k + 1) * cw]
                    d = analyse(c, bg)
                    if fr == 'bind':
                        if sname == 'walk':
                            res[k][f'stand-{vname}'] = d; cells[k][f'stand-{vname}'] = (c, bg)
                    else:
                        res[k].setdefault(f'{sname}-{vname}', []).append(d)
                        if fr in ('1', '4'): cells[k][f'{sname}{fr}-{vname}'] = (c, bg)
    return res, cells


def summarise(r):
    """one dict per entry: the median over all 34 pictures (2 standing + 32 moving), and standing / moving apart"""
    allf = [r['stand-front'], r['stand-back']] + r['walk-front'] + r['walk-back'] + r['run-front'] + r['run-back']
    mov = r['walk-front'] + r['walk-back'] + r['run-front'] + r['run-back']
    out = {}
    for k in KEYS:
        out[k] = round(float(np.median([d[k] for d in allf])), 2)
        out[k + '_stand'] = round(float(np.mean([r['stand-front'][k], r['stand-back'][k]])), 2)
        out[k + '_move'] = round(float(np.median([d[k] for d in mov])), 2)
        out[k + '_min'] = round(float(min(d[k] for d in allf)), 2); out[k + '_max'] = round(float(max(d[k] for d in allf)), 2)
        out[k + '_front'] = round(float(np.median([d[k] for d in [r['stand-front']] + r['walk-front'] + r['run-front']])), 2)
        out[k + '_back'] = round(float(np.median([d[k] for d in [r['stand-back']] + r['walk-back'] + r['run-back']])), 2)
    return out
