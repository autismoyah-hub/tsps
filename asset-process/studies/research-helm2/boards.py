"""boards.py -> boards/board-1-sharp-folds.png, board-2-stripes-dark-side.png, board-3-proportion-outfit.png
Umbral A beside Meridian v5 (passed), Torva and the test helm A2, with each separating difference made visible."""
import sys, pathlib, json
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
from scipy import ndimage
from PIL import Image, ImageDraw, ImageFont
import h2, measures as M

D = json.load(open(h2.HERE / 'data' / 'measures.json'))
OUT = h2.HERE / 'boards'; OUT.mkdir(exist_ok=True)
KEYS = [k for k in ['umbralA', 'v5', 'torva', 'oathplate', 'umbralA2m'] if k in D]
font = lambda s, b=False: ImageFont.truetype('segoeui' + ('b' if b else '') + '.ttf', s)
BG = '#1c2226'


def title(c, lines):
    d = ImageDraw.Draw(c)
    d.text((16, 10), lines[0], font=font(26, True), fill='#f2f0e9')
    for n, l in enumerate(lines[1:]): d.text((17, 48 + n * 22), l, font=font(15), fill='#b6c6c7')
    return d


# ---------------------------------------------------------------------------------------------- 1 folds
def fold_view(key, view, size=330, ppu=6.0):
    mh = h2.mesh(key); T, n, area, ed, col, idx = M.edges_of(mh)
    shell = np.array([not M.fitlib.wearer_face(mh['col'][i], mh['raw'][i]) for i in idx])
    cen = T.mean(1); out = cen - np.array([0, 182.0, -8.0]); nn = np.where(((n * out).sum(1) < 0)[:, None], -n, n)
    if view == 'front': u = lambda p: (size / 2 + p[0] * ppu, size * 0.86 - (p[1] - 163) * ppu); depth = -cen[:, 2]; facing = nn[:, 2] < 0.15
    else: u = lambda p: (size / 2 + (p[2] + 8) * ppu, size * 0.86 - (p[1] - 163) * ppu); depth = cen[:, 0]; facing = nn[:, 0] > -0.15
    im = Image.new('RGB', (size, size), (38, 34, 30)); d = ImageDraw.Draw(im)
    for i in np.argsort(depth):
        if not shell[i]: continue
        g = 40 + int(col[i][2] * 0.9)
        d.polygon([u(p) for p in T[i]], fill=(g, g, g))
    vis = lambda i: shell[i] and facing[i]
    # crude visibility: an edge is drawn if one of its faces looks at the camera and it lies on the near half
    lines = []
    for e, fs in ed.items():
        if len(fs) != 2: continue
        (i, ln), (j, _) = fs
        if not (shell[i] and shell[j]) or not (facing[i] or facing[j]): continue
        ang = np.degrees(np.arccos(np.clip(n[i] @ n[j], -1, 1)))
        # the edge's end points: shared vertices of the two triangles
        pts = [p for p in T[i] if any(np.allclose(p, q) for q in T[j])]
        if len(pts) != 2: continue
        if max(depth[i], depth[j]) < np.median(depth) - 2: continue
        lines.append((ang, pts))
    for ang, pts in sorted(lines, key=lambda x: x[0]):
        if ang > 45: d.line([u(pts[0]), u(pts[1])], fill=(255, 150, 40), width=3)
        elif ang >= 30: d.line([u(pts[0]), u(pts[1])], fill=(230, 220, 90), width=1)
        else: d.line([u(pts[0]), u(pts[1])], fill=(105, 115, 125), width=1)
    return im


def board1():
    W = 330
    c = Image.new('RGB', (W * len(KEYS), 110 + 2 * W + 120), BG)
    d = title(c, ['1  Sharp: how much of the helm\'s edge length is a hard fold',
                  'Orange = folded more than 45 degrees. Yellow = 30 to 45 (neither a plane nor a crease). Grey = softer. Own drawing of each worn mesh, front and side.',
                  'Accepted six: 0.254 to 0.447 of the shell\'s edge length is orange (skull alone 0.241 to 0.488). All seven rejected helms: 0.028 to 0.213.'])
    for i, k in enumerate(KEYS):
        c.paste(fold_view(k, 'front'), (i * W, 110)); c.paste(fold_view(k, 'side'), (i * W, 110 + W))
        r = D[k]; y = 110 + 2 * W + 6
        d.text((i * W + 10, y), r['name'], font=font(17, True), fill='#f2f0e9')
        bad = lambda v, lo: '#ff8c5a' if v < lo else '#9fe0a0'
        d.text((i * W + 10, y + 26), f"hard folds, whole shell  {r['fold45_share']:.3f}", font=font(16), fill=bad(r['fold45_share'], 0.254))
        d.text((i * W + 10, y + 48), f"hard folds, skull alone  {r['fold45_dome']:.3f}", font=font(16), fill=bad(r['fold45_dome'], 0.241))
        d.text((i * W + 10, y + 70), f"{r['faces']} faces; firm 30-45: {r['firm_share']:.3f}", font=font(14), fill='#b6c6c7')
    c.save(OUT / 'board-1-sharp-folds.png'); print(OUT / 'board-1-sharp-folds.png', c.size)


# ---------------------------------------------------------------------------------------------- 2 stripes / dark side
def board2():
    es = []
    for k in KEYS: es += [h2.entry(k), h2.entry(k, helm=False)]
    cw, ch = 300, 300
    rows = [(0, 'front'), (512, 'side (unlit)'), (1536, 'side (lit)')]
    c = Image.new('RGB', (2 * cw * len(KEYS), 110 + len(rows) * ch + 130), BG)
    d = title(c, ['2  Stripes and the dark side',
                  'Left of each pair: the client\'s picture, head close. Right: what the measure sees: RED = a level tone step (a band across the helm), CYAN = an upright step, MAGENTA = the largest near-black patch.',
                  'Accepted six at game size: level bands / upright bands in the side view 0.87 to 1.44; largest black patch in the side view 0.02 to 0.19 of the helm; side and back steel painted under lightness 20: 0 to 0.34.'])
    for ri, (yaw, cap) in enumerate(rows):
        a = h2.draw(f'b2-{yaw}', es, yaw=yaw, zoom=130, cell=(cw, ch), aim='head')
        cs = h2.cells(a, len(es))
        for i, k in enumerate(KEYS):
            w, wo = cs[2 * i], cs[2 * i + 1]
            cell, Mk, bg = M.helm_cell(w, wo); G = h2.grey(w)
            y = 110 + ri * ch
            c.paste(Image.fromarray(w.astype(np.uint8)), (2 * i * cw, y))
            v = np.zeros(w.shape, np.uint8); v[:] = (30, 28, 26); v[Mk] = (np.clip(G[Mk], 0, 255) * 0.45 + 25)[:, None].astype(np.uint8)
            lev = np.zeros_like(Mk); lev[:-1] = Mk[:-1] & Mk[1:] & (np.abs(G[:-1] - G[1:]) >= 24)
            upr = np.zeros_like(Mk); upr[:, :-1] = Mk[:, :-1] & Mk[:, 1:] & (np.abs(G[:, :-1] - G[:, 1:]) >= 24)
            dk = Mk & (G < 16); lab, n = ndimage.label(dk)
            if n:
                sz = ndimage.sum(dk, lab, range(1, n + 1)); big = lab == (1 + int(np.argmax(sz)))
                if yaw != 0: v[big] = (150, 40, 150)
            v[upr] = (60, 220, 230); v[lev] = (240, 60, 50)
            c.paste(Image.fromarray(v), ((2 * i + 1) * cw, y))
            d.text((2 * i * cw + 8, y + 4), cap, font=font(14), fill='#d8d8c8')
    y = 110 + len(rows) * ch + 6
    for i, k in enumerate(KEYS):
        r = D[k]; x = 2 * i * cw + 10
        col = lambda v, hi: '#ff8c5a' if v > hi else '#9fe0a0'
        d.text((x, y), r['name'], font=font(17, True), fill='#f2f0e9')
        d.text((x, y + 26), f"side: level / upright bands  {r['g_side_band_ratio']}", font=font(16), fill=col(r['g_side_band_ratio'], 1.44))
        d.text((x, y + 48), f"front: level / upright steps  {r['g_front_level_ratio']}", font=font(16), fill=col(r['g_front_level_ratio'], 0.95))
        d.text((x, y + 70), f"side: largest black patch  {r['g_side_void_big']}", font=font(16), fill=col(r['g_side_void_big'], 0.189))
        d.text((x, y + 92), f"side/back steel under L20  {r['bk_sideback_under20']}", font=font(16), fill=col(r['bk_sideback_under20'], 0.341))
    c.save(OUT / 'board-2-stripes-dark-side.png'); print(OUT / 'board-2-stripes-dark-side.png', c.size)


# ---------------------------------------------------------------------------------------------- 3 proportion / outfit
def board3():
    keys = [k for k in ['umbralA', 'umbralAm', 'umbralA2', 'umbralA2m', 'v5', 'torva', 'oathplate'] if k in D]
    es = []
    for k in keys: es += [h2.entry(k), h2.entry(k, helm=False)]
    cw, ch = 240, 430
    a = h2.draw('b3', es, yaw=0, zoom=300, cell=(cw, 450))
    cs = h2.cells(a, len(es))
    c = Image.new('RGB', (cw * len(keys), 132 + ch + 250), BG)
    d = title(c, ['3  Proportion, and the helm against its own body',
                  'Whole figure at game size. Yellow box: the helm\'s whole height against the width of its shell at the skull (accepted six: height / width 1.40 to 1.83; rise above the crown / width 0.19 to 0.53).',
                  'Bars under each figure: tone range (5th to 95th percentile of grey) of the HELM and of the BODY just under it. Accepted six: the helm\'s range is 0.90 to 2.04 of its body\'s, its lights no more',
                  'than 16 grey levels under the body\'s lights, and the body\'s range is 63 to 122. Both Umbral helms stood on a body with a range of 138 to 150: the stand-in outshouts the helm.'])
    for i, k in enumerate(keys):
        w, wo = cs[2 * i], cs[2 * i + 1]; r = D[k]
        cell, Mk, bg = M.helm_cell(w, wo)
        im = Image.fromarray(w.astype(np.uint8)[:ch]); c.paste(im, (i * cw, 132))
        ys, xs = np.nonzero(Mk)
        ppu = 512 / 300.0
        cx = xs.mean(); wsk = r['fit_w_skull'] * ppu * 1.0
        d.rectangle([i * cw + cx - wsk / 2, 132 + ys.min(), i * cw + cx + wsk / 2, 132 + ys.max()], outline=(240, 210, 60))
        y = 132 + ch + 8
        d.text((i * cw + 8, y), r['name'], font=font(14, True), fill='#f2f0e9')
        col = lambda bad: '#ff8c5a' if bad else '#9fe0a0'
        d.text((i * cw + 8, y + 22), f"height / skull width {r['pr_h_over_wskull']}", font=font(14), fill=col(r['pr_h_over_wskull'] > 1.83))
        d.text((i * cw + 8, y + 40), f"rise / skull width {r['pr_above_over_wskull']}", font=font(14), fill=col(r['pr_above_over_wskull'] > 0.53))
        # tone bars
        y2 = y + 70
        for n, (lab, lo, hi, colr) in enumerate((('helm', r['g_front_p05'], r['g_front_p95'], (200, 200, 210)),
                                                  ('body', r['g_front_body_p50'] - 0, None, (150, 160, 150)))):
            if lab == 'body':
                hi = r['g_front_p95'] - r['g_front_p95_minus_body_p95']; lo = hi - r['g_front_body_range']
            x0 = i * cw + 50; s = 0.85
            d.text((i * cw + 8, y2 + n * 26), lab, font=font(13), fill='#b6c6c7')
            d.rectangle([x0, y2 + n * 26 + 3, x0 + 255 * s, y2 + n * 26 + 17], outline=(70, 70, 70))
            d.rectangle([x0 + lo * s, y2 + n * 26 + 3, x0 + hi * s, y2 + n * 26 + 17], fill=colr)
        d.text((i * cw + 8, y2 + 58), f"helm range / body range {r['g_front_range_vs_body']}", font=font(14), fill=col(r['g_front_range_vs_body'] < 0.9))
        d.text((i * cw + 8, y2 + 76), f"helm lights - body lights {r['g_front_p95_minus_body_p95']}", font=font(14), fill=col(r['g_front_p95_minus_body_p95'] < -16.3))
        d.text((i * cw + 8, y2 + 94), f"body range {r['g_front_body_range']}", font=font(14), fill=col(r['g_front_body_range'] > 122))
    c.save(OUT / 'board-3-proportion-outfit.png'); print(OUT / 'board-3-proportion-outfit.png', c.size)


if __name__ == '__main__':
    which = sys.argv[1:] or ['1', '2', '3']
    if '1' in which: board1()
    if '2' in which: board2()
    if '3' in which: board3()
