"""board.py -> board-helm3.png: Umbral C, Umbral A, Meridian v5, Torva; game-size front / three-quarter / side, close front /
three-quarter, and a drawing of each worn mesh with its hard-fold lines.  No pre-registered measure survived, so no
measure hot spots exist; the rings on C are what I SAW (by eye, written in HELM3.md before they were drawn), and the
mesh row shows the nearest miss (in_dangling) to show where it looks: it calls C tidy."""
import sys, pathlib, json
sys.dont_write_bytecode = True
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import numpy as np
from PIL import Image, ImageDraw, ImageFont
import h3
from h3 import h2

KEYS = ['umbralC', 'umbralA', 'v5', 'torva']
NAMES = ['Umbral C (rejected, worse than A)', 'Umbral A (rejected)', 'Meridian v5 (accepted)', 'Torva (accepted)']
CS = 300
font = lambda s, b=False: ImageFont.truetype('segoeui' + ('b' if b else '') + '.ttf', s)
W = h3.h2.WORK

# by-eye rings on C: row -> list of (x, y, r, label), cell coordinates (0..300)
MARKS = {
    'game front': [(150, 75, 45, '1'), (150, 165, 40, '2'), (85, 140, 28, '3')],
    'game 3/4':   [(120, 60, 45, '1'), (170, 170, 45, '4'), (75, 175, 30, '2')],
    'game side':  [(110, 55, 50, '1'), (95, 150, 40, '5')],
    'close front': [(150, 85, 50, '1'), (150, 185, 45, '2'), (80, 155, 30, '3'), (150, 135, 40, '6')],
    'close 3/4':  [(130, 75, 55, '1'), (190, 185, 55, '4'), (85, 200, 35, '2')],
}


def crop(tag, k, box, scale):
    im = Image.open(W / f'{tag}.png').convert('RGB'); cw = im.width // 4
    x0, y0, x1, y1 = box
    return im.crop((k * cw + x0, y0, k * cw + x1, y1)).resize((CS, CS), Image.NEAREST)


def fold_panel(key, size=CS, ppu=6.5):
    mh = h2.mesh(key); P, F = h3.shell(mh); T, n, area, ed, fold, ln = h3.topo(P, F)
    cl, dang, tot, at = h3.chains(P, ed, fold, ln)
    # chain membership for colouring: recompute lengths per edge via the same linking
    hard = [e for e, a in fold.items() if a > 35]
    boundary = set()
    for e, fs in ed.items():
        if len(fs) == 1: boundary |= set(e)
    u = lambda p: (size / 2 + p[0] * ppu, size * 0.88 - (p[1] - 166) * ppu)
    im = Image.new('RGB', (size, size), (38, 34, 30)); d = ImageDraw.Draw(im)
    cen = T.mean(1)
    front = n[:, 2] < 0.1
    for i in np.argsort(-cen[:, 2])[::-1][::-1]:
        pass
    for i in np.argsort(cen[:, 2])[::-1]:      # far (z large = back) first
        g = 70 + int(60 * max(0.0, -n[i, 2])) if front[i] else 45
        d.polygon([u(p) for p in T[i]], fill=(g, g, g + 6))
    for e in hard:
        fs = ed[e]
        if not any(front[f] for f in fs): continue
        d.line([u(P[e[0]]), u(P[e[1]])], fill=(255, 150, 40), width=2)
    for v, es in at.items():
        if len(es) == 1 and v not in boundary and P[v, 2] < 2:
            x, y = u(P[v]); d.ellipse((x - 4, y - 4, x + 4, y + 4), outline=(255, 60, 60), width=2)
    d.text((6, size - 22), f'dangling ends / 100u: {100 * dang / tot:.2f}', font=font(14), fill='#f2f0e9')
    return im


if __name__ == '__main__':
    rows = [('game front', lambda k: crop('look-g-0', k, (35, 25, 135, 125), 3)),
            ('game 3/4', lambda k: crop('look-g-256', k, (35, 25, 135, 125), 3)),
            ('game side', lambda k: crop('look-g-512', k, (35, 25, 135, 125), 3)),
            ('close front', lambda k: crop('look-c-0', k, (40, 20, 280, 260), 1.25)),
            ('close 3/4', lambda k: crop('look-c-256', k, (40, 20, 280, 260), 1.25)),
            ('mesh: hard folds (orange), dangling ends (red)', lambda k: fold_panel(KEYS[k]))]
    top, left = 120, 150
    c = Image.new('RGB', (left + CS * 4, top + CS * len(rows)), '#1c2226'); d = ImageDraw.Draw(c)
    d.text((14, 8), 'HELM3 board: C beside A, Meridian v5 and Torva', font=font(26, True), fill='#f2f0e9')
    d.text((15, 46), 'No pre-registered measure survived (0 of 17), so there are no measure hot spots. Rings on C = by eye: '
           '1 lopsided crescent, 2 egg-like face lump,', font=font(15), fill='#b6c6c7')
    d.text((15, 68), '3 dark wedge breaking the cheek outline, 4 patchwork of small unequal facets on temple/cheek, 5 kinked '
           'side profile, 6 eye slits not seated in a frame.', font=font(15), fill='#b6c6c7')
    for k, nm in enumerate(NAMES): d.text((left + k * CS + 8, top - 28), nm, font=font(16, True), fill='#f2f0e9')
    for r, (name, fn) in enumerate(rows):
        d.text((8, top + r * CS + 10), name.replace(': ', ':\n').replace(', ', ',\n'), font=font(15), fill='#b6c6c7')
        for k in range(4):
            im = fn(k)
            if k == 0 and name in MARKS:
                dd = ImageDraw.Draw(im)
                for x, y, rr, lab in MARKS[name]:
                    dd.ellipse((x - rr, y - rr, x + rr, y + rr), outline=(90, 230, 255), width=2)
                    dd.text((x + rr * 0.7, y - rr * 0.9), lab, font=font(18, True), fill=(90, 230, 255))
            c.paste(im, (left + k * CS, top + r * CS))
    c.save(HERE / 'board-helm3.png'); print('board-helm3.png', c.size)
