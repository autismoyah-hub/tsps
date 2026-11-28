"""Test of predict_light.py against the renderer: compare its prediction with line-up captures of the same model.

usage: check_predict.py MODEL.bin LINEUP_DIR ENTRY --ambient A --contrast C --dist D --elev E [--up U] [--views N] [--save]
  LINEUP_DIR  a folder made by lineup.py; ENTRY = position of MODEL.bin in its entry list (0 = first)
  give the same --ambient/--contrast/--dist/--elev the line-up was made with
  --save      also write LINEUP_DIR/predicted_vR.png (capture | prediction | where they differ)

Two measurements per view:
  faces   at the screen centroid of every face that shows a clear patch there (the 5x5 pixels around the
          centroid all belong to it in the prediction): captured lightness minus predicted lightness, where the
          captured lightness is read by exact match in the colour table at the face's own hue and saturation
  pixels  every pixel where the prediction shows the model: share whose RGB equals the capture exactly, and
          within 1 and 2 lightness steps
"""
import sys, pathlib, argparse
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import numpy as np
from PIL import Image
import gamecol, predict_light as pl

ap = argparse.ArgumentParser()
ap.add_argument('model'); ap.add_argument('dir'); ap.add_argument('entry', type=int)
ap.add_argument('--ambient', type=int, required=True); ap.add_argument('--contrast', type=int, required=True)
ap.add_argument('--dist', type=float, default=330); ap.add_argument('--elev', type=float, default=250)
ap.add_argument('--up', type=float, default=45); ap.add_argument('--views', type=int, default=4)
ap.add_argument('--save', action='store_true')
a = ap.parse_args()
m = pl.load(a.model)
lit = pl.light(m, a.ambient, a.contrast)
pal = gamecol.table()
flat = np.array(m['render_types']) == 1
for v in range(a.views):
    cap = np.asarray(Image.open(f'{a.dir}/e{a.entry}v{v}.png').convert('RGB')).astype(np.int32)
    img, who = pl.picture(m, lit, v, a.dist, a.elev, a.up)
    x, y, _ = pl.project(m, v, a.dist, a.elev, a.up)
    errs = {False: [], True: []}; wrong_hue = 0; big = []; pic = []
    for i, f in enumerate(m['faces']):
        if lit[i] is None:
            continue
        cx, cy = int(sum(x[k] for k in f) / 3), int(sum(y[k] for k in f) / 3)
        if not (2 <= cx < pl.W - 2 and 2 <= cy < pl.H - 2) or not (who[cy - 2:cy + 3, cx - 2:cx + 3] == i).all():
            continue
        base = m['colors'][i] & 65408
        line = pal[base:base + 128]
        hit = np.nonzero((line == cap[cy, cx]).all(axis=1))[0]
        if len(hit) == 0:
            wrong_hue += 1; continue
        want = sum(c & 127 for c in lit[i]) / 3
        got = hit[np.abs(hit - want).argmin()]                 # equal RGBs at neighbouring lightness: take the nearest
        errs[bool(flat[i])].append(got - want)
        mine = np.nonzero((line == img[cy, cx]).all(axis=1))[0]
        pic.append(hit[np.abs(hit - mine[0]).argmin()] - mine[0])
        if not flat[i] and (who == i).sum() >= 400:
            big.append(got - want)
    on = who >= 0
    d = np.abs(cap - img.astype(np.int32)).sum(axis=2)
    # lightness distance per pixel: capture's lightness on the predicted face's hue/sat line
    base = (np.array(m['colors'])[who[on]] & 65408)
    pred_l = np.zeros(on.sum(), dtype=int)
    capl = np.full(on.sum(), -99)
    cp = cap[on]
    pi = img[on].astype(np.int32)
    for l in range(128):
        row = pal[base + l]
        capl = np.where((capl == -99) & (row == cp).all(axis=1), l, capl)
        pred_l = np.where((row == pi).all(axis=1) & (pred_l == 0), l, pred_l)
    known = capl != -99
    dl = np.abs(capl - pred_l)[known]
    print(f'view {v}:')
    for k, name in ((False, 'smooth'), (True, 'flat')):
        e = np.array(errs[k])
        if len(e):
            print(f'   faces {name:6}: {len(e):4d} sampled; captured - predicted lightness: mean {e.mean():+.2f}, mean abs {np.abs(e).mean():.2f}, '
                  f'max abs {np.abs(e).max():.1f}; within 1 step {100 * (np.abs(e) <= 1).mean():.0f}%')
        else:
            print(f'   faces {name:6}: none with a clear patch')
    if big:
        e = np.array(big)
        print(f'   smooth faces of 400+ pixels only: {len(e)} sampled; mean {e.mean():+.2f}, mean abs {np.abs(e).mean():.2f}, max abs {np.abs(e).max():.1f}')
    if pic:
        e = np.array(pic)
        print(f'   same samples against the PICTURE at that pixel (4-pixel shading steps included): mean {e.mean():+.2f}, mean abs {np.abs(e).mean():.2f}, max abs {np.abs(e).max()}')
    print(f'   faces whose sampled pixel is not on their own hue/sat line (another face drawn there): {wrong_hue}')
    print(f'   pixels: {on.sum()} predicted on the model; identical RGB {100 * (d[on] == 0).mean():.1f}%; same hue/sat line {100 * known.mean():.1f}%, '
          f'of those within 1 lightness step {100 * (dl <= 1).mean():.1f}%, within 2 {100 * (dl <= 2).mean():.1f}%, mean abs {dl.mean():.2f}')
    if a.save:
        out = np.zeros((pl.H, pl.W * 3, 3), dtype=np.uint8)
        img2 = img.copy(); img2[~on] = cap[~on]
        out[:, :pl.W] = cap; out[:, pl.W:2 * pl.W] = img2
        diff = np.full((pl.H, pl.W, 3), 40, dtype=np.uint8)
        diff[on & (d == 0)] = (90, 90, 90); diff[on & (d > 0)] = (255, 200, 0); diff[on & (d > 24)] = (255, 0, 0)
        out[:, 2 * pl.W:] = diff
        c0, c1 = pl.W // 2 - 300, pl.W // 2 + 300
        cut = np.concatenate([out[150:750, c0:c1], out[150:750, pl.W + c0:pl.W + c1], out[150:750, 2 * pl.W + c0:2 * pl.W + c1]], axis=1)
        Image.fromarray(cut).save(f'{a.dir}/predicted_v{v}.png')
