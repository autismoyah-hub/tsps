"""tsps_compare.py SHOT.png TSPS_X TSPS_Y STATE.jsonl ROOT_UID OUT_PREFIX [--cache 242]

Compare the TSPS client's own drawing of an interface (a headless screenshot) with iface.py's drawing of the same
laid-out state. The interface's root component ROOT_UID sits at (TSPS_X, TSPS_Y) in the screenshot (read from the
running client) and wherever the state puts it in ours; both are cropped to the root's rectangle. Pixels under
components iface.py does not draw (models, item icons) are excluded and counted separately. Writes
OUT_PREFIX-board.png (TSPS | ours | difference) and prints a JSON summary."""
import sys, json, argparse
from pathlib import Path
import numpy as np
from PIL import Image

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import ifcache as C, iface


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('shot'); ap.add_argument('x', type=int); ap.add_argument('y', type=int)
    ap.add_argument('state'); ap.add_argument('root'); ap.add_argument('out')
    ap.add_argument('--cache', default='242')
    ap.add_argument('--exclude', default='', help='extra rectangles x0,y0,x1,y1;... in root coordinates')
    ap.add_argument('--native-blend', action='store_true', help='use the software blend instead of TSPS WebGL blend')
    a = ap.parse_args()
    iface.TSPS_BLEND = not a.native_blend
    cache = C.Cache(C.CACHE_242 if a.cache == '242' else C.CACHE_239)
    g, c = (int(v) for v in a.root.split('.'))
    root = (g << 16) | c
    order, lay, byid = iface.load_state(a.state)
    ras, rep = iface.draw(cache, order, lay, byid, iface.Raster(1280, 800), (1280, 800))
    ours = ras.rgb()
    ras2, _ = iface.draw(cache, order, lay, byid, iface.Raster(1280, 800, 0xff00ff00), (1280, 800))
    rx, ry = lay[root]['abs']; w, h = lay[root]['size']
    mine = ours[ry:ry + h, rx:rx + w].astype(int)
    # pixels our drawing does not fully cover (the world or a lower layer shows through): not comparable
    through = (ras2.rgb()[ry:ry + h, rx:rx + w].astype(int) != mine).any(-1)
    shot = np.asarray(Image.open(a.shot).convert('RGB')).astype(int)
    theirs = shot[a.y:a.y + h, a.x:a.x + w]
    mask = np.zeros((h, w), bool)
    for i, why in rep['not_drawn']:
        if why.startswith('special content'):
            continue
        cl = lay[i]['clip']
        x0, y0, x1, y1 = cl[0] - rx, cl[1] - ry, cl[2] - rx, cl[3] - ry
        mask[max(y0, 0):max(y1, 0), max(x0, 0):max(x1, 0)] = True
    for r in filter(None, a.exclude.split(';')):
        x0, y0, x1, y1 = (int(v) for v in r.split(','))
        mask[y0:y1, x0:x1] = True
    see_through = through & ~mask
    mask |= through
    d = (mine != theirs).any(-1)
    delta = np.abs(mine - theirs).max(-1)
    out = dict(root=a.root, size=[w, h], pixels=int(w * h), excluded=int(mask.sum()), see_through=int(see_through.sum()),
               differ=int((d & ~mask).sum()), differ_in_excluded=int((d & mask).sum()),
               max_delta=int(delta[~mask].max()) if (~mask).any() else 0,
               differ_delta_gt_2=int(((delta > 2) & ~mask).sum()))
    if out['differ']:
        ys, xs = np.nonzero(d & ~mask)
        out['box'] = [int(xs.min()), int(ys.min()), int(xs.max()), int(ys.max())]
    vis = (mine * 0.35).astype(np.uint8)
    vis[d & ~mask] = [255, 0, 255]
    vis[mask] = (vis[mask] * 0.5 + np.array([0, 60, 120])).astype(np.uint8)
    pad = 6
    board = np.full((h, w * 3 + pad * 2, 3), 20, np.uint8)
    board[:, :w] = theirs.astype(np.uint8); board[:, w + pad:2 * w + pad] = mine.astype(np.uint8)
    board[:, 2 * w + 2 * pad:] = vis
    Image.fromarray(board).save(a.out + '-board.png')
    Image.fromarray(theirs.astype(np.uint8)).save(a.out + '-tsps.png')
    Image.fromarray(mine.astype(np.uint8)).save(a.out + '-ours.png')
    print(json.dumps(out))


if __name__ == '__main__':
    main()
