"""palette.py: what the pictures say: the floor's colours against the boss's, measured on the straight-down view.

  measure(top_png, empty_png, P, M, Z, hfun) -> dict
    top_png / empty_png: the same camera with and without the boss; P: draw.project(...) of that camera;
    M: the arena model (floor, boss); Z: zone of each hull tile (arenamodel.zones); hfun(x, y) corner height.
The boss is every pixel that differs between the two pictures. Colour distance is CIE76 delta E (Lab, D65).
"""
import math, numpy as np
from PIL import Image, ImageDraw, ImageFilter


def lab(rgb):
    c = np.asarray(rgb, dtype=np.float64) / 255
    c = np.where(c > 0.04045, ((c + 0.055) / 1.055) ** 2.4, c / 12.92)
    X = (c[..., 0] * 0.4124 + c[..., 1] * 0.3576 + c[..., 2] * 0.1805) / 0.95047
    Y = c[..., 0] * 0.2126 + c[..., 1] * 0.7152 + c[..., 2] * 0.0722
    Zc = (c[..., 0] * 0.0193 + c[..., 1] * 0.1192 + c[..., 2] * 0.9505) / 1.08883
    f = lambda t: np.where(t > 0.008856, np.cbrt(t), 7.787 * t + 16 / 116)
    return np.stack([116 * f(Y) - 16, 500 * (f(X) - f(Y)), 200 * (f(Y) - f(Zc))], -1)


def hue_name(rgb):
    import colorsys
    h, l, s = colorsys.rgb_to_hls(*[v / 255 for v in rgb])
    if s < 0.12: return 'grey' if 0.2 < l < 0.8 else ('black' if l <= 0.2 else 'white')
    names = ['red', 'orange', 'yellow', 'green', 'green', 'cyan', 'blue', 'blue', 'purple', 'magenta', 'red']
    n = names[int(h * 360 / 36)]
    if n in ('orange', 'yellow', 'red') and l < 0.35: n = 'brown' if n != 'red' else 'dark red'
    return n


def mask_tiles(size, P, tiles, hfun):
    im = Image.new('L', size, 0); dr = ImageDraw.Draw(im)
    for x, y in tiles:
        pts = [P(a, b, hfun(a, b)) for a, b in ((x, y), (x + 1, y), (x + 1, y + 1), (x, y + 1))]
        if all(pts): dr.polygon(pts, fill=255)
    return np.asarray(im) > 0


def stats(px):
    """px: N x 3 uint8"""
    if len(px) == 0: return None
    f = px.astype(np.float64)
    luma = f @ np.array([0.299, 0.587, 0.114])
    mx = f.max(1); mn = f.min(1); sat = np.where(mx > 0, (mx - mn) / np.maximum(mx, 1), 0)
    mean = f.mean(0)
    return dict(rgb=[int(round(v)) for v in mean], luma=float(luma.mean()), luma_sd=float(luma.std()), sat=float(sat.mean()),
                hue=hue_name(mean), L=float(lab(mean)[0]))


def n_colours(px, share=0.9):
    """Colours needed for `share` of the pixels, colours counted in bins of 32 levels a channel (512 bins)."""
    if len(px) == 0: return 0
    q = (px // 32).astype(np.int32); k = q[:, 0] * 64 + q[:, 1] * 8 + q[:, 2]
    c = np.sort(np.bincount(k, minlength=512))[::-1]; acc = np.cumsum(c)
    return int(np.searchsorted(acc, share * len(px)) + 1)


def measure(top_png, empty_png, P, M, Z, hfun):
    a = np.asarray(Image.open(top_png).convert('RGB')); e = np.asarray(Image.open(empty_png).convert('RGB'))
    size = (a.shape[1], a.shape[0])
    S = M['floor']; H = set(Z)
    floor = mask_tiles(size, P, S, hfun)
    m = {}
    fs = stats(e[floor]); m['floor'] = fs
    m['floor_colours_90'] = n_colours(e[floor])
    zs = {}
    for z in ('centre', 'middle', 'band'):
        zm = mask_tiles(size, P, [t for t in S if Z[t] == z], hfun)
        s = stats(e[zm]); m['floor_' + z + '_luma'] = s['luma'] if s else None; zs[z] = s
    de = lambda p, q: float(np.linalg.norm(lab(np.array(p['rgb'])) - lab(np.array(q['rgb'])))) if p and q else None
    m['centre_vs_middle_dE'] = de(zs['centre'], zs['middle'])       # is the centre of the floor marked?
    m['band_vs_middle_dE'] = de(zs['band'], zs['middle'])           # is the border of the floor marked?
    fp = e[floor]
    if len(fp):
        med = np.median(fp, axis=0)
        m['floor_plain_share'] = float((np.linalg.norm(lab(fp) - lab(med), axis=1) <= 10).mean())   # share of the floor within delta E 10 of its median colour
    # the surround: two tiles outside the outline
    ring = {(t[0] + i, t[1] + j) for t in H for i in (-2, -1, 0, 1, 2) for j in (-2, -1, 0, 1, 2)} - H
    sm = mask_tiles(size, P, ring, hfun) & ~floor
    ss = stats(e[sm]); m['surround'] = ss
    # the boss: pixels that differ between the two pictures
    diff = np.abs(a.astype(np.int32) - e.astype(np.int32)).max(2) > 14
    bm = np.asarray(Image.fromarray((diff * 255).astype(np.uint8)).filter(ImageFilter.MedianFilter(3))) > 0
    m['boss_pixels'] = int(bm.sum())
    if bm.sum() >= 40:
        bs = stats(a[bm]); m['boss'] = bs
        grow = np.asarray(Image.fromarray((bm * 255).astype(np.uint8)).filter(ImageFilter.MaxFilter(51))) > 0
        local = grow & ~bm & floor
        if local.sum() < 200: local = grow & ~bm
        ls = stats(e[local]); m['under_boss'] = ls
        m['luma_contrast'] = bs['luma'] - ls['luma']
        m['L_contrast'] = bs['L'] - ls['L']
        de = np.linalg.norm(lab(np.array(bs['rgb'])) - lab(np.array(ls['rgb'])))
        m['deltaE_mean'] = float(de)
        pix = lab(a[bm]); ref = lab(np.array(ls['rgb']))
        d = np.linalg.norm(pix - ref, axis=1)
        m['boss_far_share'] = float((d > 20).mean())       # share of the boss's pixels clearly off the floor's colour
        m['sat_contrast'] = bs['sat'] - ls['sat']
        m['boss_colours_90'] = n_colours(a[bm])
    # where the accent sits: the 2% most vivid pixels (saturation x value) of floor + surround
    allm = floor | sm
    f = e.astype(np.float64); mx = f.max(2); mn = f.min(2)
    viv = np.where(mx > 0, (mx - mn) / np.maximum(mx, 1), 0) * mx / 255
    v = viv[allm]
    if v.size:
        thr = np.quantile(v, 0.98); top = (viv >= max(thr, 0.12)) & allm
        tot = max(1, top.sum()); acc = {}
        for z in ('centre', 'middle', 'band'):
            zm = mask_tiles(size, P, [t for t in H if Z[t] == z], hfun)
            acc[z] = float((top & zm).sum() / tot)
        acc['surround'] = float((top & sm & ~mask_tiles(size, P, H, hfun)).sum() / tot)
        m['accent'] = acc; m['accent_strength'] = float(thr)
    # brightest thing: mean luma of the top 2% brightest pixels and where they are
    lum = f @ np.array([0.299, 0.587, 0.114])
    lt = np.quantile(lum[allm], 0.98); topb = (lum >= lt) & allm
    m['bright_in_surround'] = float((topb & sm).sum() / max(1, topb.sum()))
    return m
