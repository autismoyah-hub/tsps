"""The three boards of CLEAN.md, real beside ours with the offenders marked.
usage: boards.py [1] [2] [3]      -> boards/board-1-splotch.png, board-2-defined.png, board-3-construction.png"""
import sys, pathlib, collections
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import numpy as np
from PIL import Image, ImageDraw, ImageFont
import cleanlib as cl, cleancheck as cc, features as ft
W = cl.B
OUT = HERE / 'boards'; OUT.mkdir(exist_ok=True)
REP = 'def:' + str(W / 'research-paint' / 'repaint' / 'watcher-2-repaint.json')
HOUND = 'def:' + str(W / 'creature-star-hound-3' / 'creature-star-hound-3.json')
WG = 'def:' + str(W / 'boss-warden-general-3' / 'boss-warden-general-3.json')
def font(n):
    try: return ImageFont.truetype('segoeui.ttf', n)
    except Exception: return ImageFont.load_default()

def box_of(c, up, zfrac=None, xfrac=None):
    skin = c['drawn'] & (c['A'] == 0); Vu = c['V'][np.unique(c['F'][skin])]; lo, hi = Vu.min(0), Vu.max(0); h = hi[1] - lo[1]
    y0, y1 = hi[1] - up[1] * h, hi[1] - up[0] * h
    sel = Vu[(Vu[:, 1] >= y0) & (Vu[:, 1] <= y1)]
    if zfrac: z0, z1 = lo[2] + zfrac[0] * (hi[2] - lo[2]), lo[2] + zfrac[1] * (hi[2] - lo[2]); sel = sel[(sel[:, 2] >= z0) & (sel[:, 2] <= z1)]
    b0, b1 = sel.min(0), sel.max(0)
    if xfrac: cx = (b0[0] + b1[0]) / 2; hw = (b1[0] - b0[0]) / 2 * xfrac; b0[0], b1[0] = cx - hw, cx + hw
    return b0, b1

def cell(c, yaw, box, size, tint=None, lines=None):
    """lines: list of (edge index into c['sev'], colour)"""
    keep = np.nonzero(c['drawn'] & (c['A'] == 0))[0]
    img, ids = cl.view(c, yaw, 8, size=size, box=box, faces=keep, tint=tint)
    canvas = np.zeros((size, size, 3), np.uint8); canvas[:] = cl.BG
    hh, ww = min(img.shape[0], size), min(img.shape[1], size); canvas[:hh, :ww] = img[:hh, :ww]
    im = Image.fromarray(canvas)
    if lines:
        X, Y, Z, W_, H_ = cl.frame(c, yaw, 8, None, size, box, keep, 8); d = ImageDraw.Draw(im); _, _, fwd = cl.camera(yaw, 8)
        for k, col in lines:
            fa, fb = c['sea'][k], c['seb'][k]; a, b = c['sev'][k]
            mx, my = int((X[a] + X[b]) / 2), int((Y[a] + Y[b]) / 2)
            if not (0 <= mx < ww and 0 <= my < hh): continue
            near = ids[max(my - 1, 0):my + 2, max(mx - 1, 0):mx + 2]
            if not (np.isin(near, (fa, fb))).any(): continue
            d.line([(X[a], Y[a]), (X[b], Y[b])], fill=col, width=2)
    return im

def caption(im, lines, size=13):
    d = ImageDraw.Draw(im); y = 3
    for i, t in enumerate(lines):
        d.text((5, y), t, fill=(240, 220, 150) if i == 0 else (235, 235, 235), font=font(size + (2 if i == 0 else 0)), stroke_width=2, stroke_fill=(0, 0, 0)); y += size + 6
    return im

def grid(cells, ncol, title, legend, out):
    w, h = cells[0].size; nrow = (len(cells) + ncol - 1) // ncol
    board = Image.new('RGB', (ncol * w, nrow * h + 62), (20, 18, 16)); d = ImageDraw.Draw(board)
    d.text((8, 4), title, fill=(240, 220, 150), font=font(19)); x = 8
    for col, txt in legend:
        d.rectangle([x, 38, x + 13, 51], fill=col); d.text((x + 18, 34), txt, fill=(225, 225, 225), font=font(13)); x += 40 + int(d.textlength(txt, font=font(13)))
    for i, cimg in enumerate(cells): board.paste(cimg, ((i % ncol) * w, 62 + (i // ncol) * h))
    board.save(out); print(out, board.size)

def board1():
    """splotch: whole figure front and back, plain and with the splotch planes in magenta"""
    cells = []
    for e, tag in (('npc:12812', 'REAL, smooth'), ('npc:7541', 'REAL, detailed'), ('npc:13599', 'REAL, detailed'), ('npc:2025', 'REAL, smooth'), (REP, 'OURS, as judged'), (str(HERE / 'watcher-clean' / 'watcher-clean.json'), 'OURS, cleaned by script'), (HOUND, 'OURS (Strong)'), (WG, 'OURS (not judged)')):
        c = cl.load(e); m, off = cc.measure(c)
        tint = {int(i): cc.MAG for i in off['splotch']}
        row = [cell(c, 25, None, 300), cell(c, 25, None, 300, tint), cell(c, 205, None, 300, tint)]
        im = Image.new('RGB', (900, 300)); [im.paste(r, (k * 300, 0)) for k, r in enumerate(row)]
        caption(im, [f"{c['name']}  [{tag}]", f"splotch {m['splotch']:.3f} of skin   (real 0.000-0.158; Minotaur 0.088, Tekton 0.098)", f"style {m['style']}: marks {m['marks']:.3f} + facets {m['facets']:.3f}",
                     f"splotch borders: {100 * m['splotch_border_paint']:.0f}% paint steps, rest light alone"], 12)
        cells.append(im)
    grid(cells, 2, 'Board 1. "Clean, smooth" against "splotchy": small weak planes with no committed border (lit as the client lights)', [(cc.MAG, 'splotch: a small plane (under 0.2% of the skin) that differs from its neighbours by under 6 lit lightness and is not bordered by a hard fold, a strong tone step or another material')], OUT / 'board-1-splotch.png')

HEADS = [('npc:12812', (0.78, 1.0), None, None, 'REAL'), ('npc:2025', (0.66, 0.86), None, 0.4, 'REAL'), ('npc:7541', (0.78, 1.0), None, None, 'REAL'), ('npc:2215', (0.82, 1.0), None, 0.5, 'REAL'),
         ('npc:15625', (0.45, 1.0), (0, 0.3), None, 'REAL'), ('npc:104', (0.5, 1.0), (0, 0.3), None, 'REAL'),
         (REP, (0.74, 1.0), None, None, 'OURS, as judged'), (str(HERE / 'watcher-clean' / 'watcher-clean.json'), (0.74, 1.0), None, None, 'OURS, cleaned by script'), (HOUND, (0.5, 1.0), (0, 0.3), None, 'OURS (Strong)'), (WG, (0.82, 1.0), None, 0.6, 'OURS (not judged)')]

def board2():
    """defined: heads with hard folds (cyan) and firm folds carrying no paint change (yellow)"""
    cells = []
    for e, up, zf, xf, tag in HEADS:
        c = cl.load(e); m, off = cc.measure(c); box = box_of(c, up, zf, xf)
        ad = np.abs(c['sdih']); hard = np.nonzero(c['sm'] & (ad >= cc.HARD))[0]
        lines = [(k, (70, 230, 255)) for k in hard] + [(k, cc.YEL) for k in off['mush_edges']]
        row = [cell(c, 30, box, 330), cell(c, 30, box, 330, None, lines), cell(c, 90, box, 330, None, lines)]
        im = Image.new('RGB', (990, 330)); [im.paste(r, (k * 330, 0)) for k, r in enumerate(row)]
        caption(im, [f"{c['name']}  [{tag}]", f"hard / firm fold length {m['hard_over_firm']:.2f}   (real 1.10-3.83; Minotaur 1.10, Tekton 1.41)", f"firm folds {m['firm_share']:.3f} of edge length (real 0.058-0.199)"], 12)
        cells.append(im)
    grid(cells, 2, 'Board 2. "Well defined": a real fold is committed. Hard folds against uncommitted firm folds, whole-creature numbers, heads shown',
         [((70, 230, 255), 'hard fold: 50 degrees or more (a crease, a ridge, a plane change)'), (cc.YEL, 'firm fold, 30-50 degrees, with no paint change: neither a crease nor smooth')], OUT / 'board-2-defined.png')

def board3():
    """construction: heads in three views with separate pieces coloured by how they are built"""
    cells = []
    for e, up, zf, xf, tag in HEADS:
        c = cl.load(e); m, off = cc.measure(c); box = box_of(c, up, zf, xf)
        feats, hgt = ft.find(c)
        tint = {}
        for f in feats:
            if f['kind'] != 'shell': continue
            col = (255, 50, 40) if (f['closed'] and f['touching']) else (60, 230, 90) if f['touching'] else (90, 150, 255)
            for i in f['fs']: tint[int(i)] = col
        row = [cell(c, 20, box, 330), cell(c, 20, box, 330, tint), cell(c, 90, box, 330, tint)]
        im = Image.new('RGB', (990, 330)); [im.paste(r, (k * 330, 0)) for k, r in enumerate(row)]
        caption(im, [f"{c['name']}  [{tag}]", f"small closed lumps {m['small_lumps']} (real 0-2)   faces in stuck-on closed pieces {m['stuck_faces']:.3f} (real 0-0.124)",
                     f"small contrasting features: {m['small_paint']} paint, {m['small_inset']} inset, {m['small_relief']} relief, {m['small_open']} open piece, {m['small_closed']} CLOSED piece",
                     f"largest welded piece {m['largest_piece']:.2f} of faces (real 0.655-1.00)"], 12)
        cells.append(im)
    grid(cells, 2, 'Board 3. "Playdough features stuck on": how eyes, nostrils, mouths and horns are built. Uncoloured = faces of the welded skin itself',
         [((255, 50, 40), 'separate CLOSED piece touching the skin (a capped lump)'), ((60, 230, 90), 'separate piece, open where it meets the skin (teeth, cards)'), ((90, 150, 255), 'separate piece not touching (floating)')], OUT / 'board-3-construction.png')

if __name__ == '__main__':
    which = sys.argv[1:] or ['1', '2', '3']
    if '1' in which: board1()
    if '2' in which: board2()
    if '3' in which: board3()
