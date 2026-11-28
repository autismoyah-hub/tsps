"""Real against ours, view by view: enlarged real | ours | signed difference, and a score per view.

usage: pairs.py LINEUP_DIR [--out DIR] [--scale 3] [--full 30] [--crop PX]
  LINEUP_DIR  a folder made by lineup.py with two entries, the real one and ours (either order; key.txt
              says which), and any number of views. Uses the per-entry captures e0vR.png / e1vR.png.
  writes      DIR/pair_vR.png, default DIR = LINEUP_DIR/pairs   (R counts from 0, like the capture files)
  --full N    grey difference that gets the full colour in the difference panel (default 30)

Difference panel: grey = same; orange to red = OURS LIGHTER than the real; light blue to blue = OURS DARKER;
green = only the real covers the pixel (ours is missing there); magenta = only ours does.
Score: mean absolute difference of grey level (0-255, mean of R, G, B) over the pixels both objects cover.
It is a DIFFERENCE, not a ratio, on purpose: on unlit faces the grey level is 20 to 40, so a ratio turns one
lightness step into a "15% error" and swamps the faces a player actually sees.

  --crop PX   centre window compared; default is the window lineup.py used for lineup.png (read from its width)

The background is found exactly: ground pixels are the ground's own hue/sat line of the colour table (the ground
being the capture's commonest colour). A part of the object painted in exactly the ground's hue and saturation
(7/0 or 8/0 on the line-up floor) would be taken for ground. Only the centre window is compared, because in the
turned views the other entry of the line-up stands near the edge of the full capture.
"""
import sys, pathlib, argparse
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import numpy as np
from PIL import Image
import gamecol


def load(path):
    return np.asarray(Image.open(path).convert('RGB')).astype(np.int32)


def object_mask(im):
    """True where the picture shows an object: not on the ground's hue/sat line of the colour table."""
    return ~gamecol.ground_mask(im)


def compare(real, ours, full=30):
    """-> (difference picture, stats dict) for two captures of the same camera."""
    ma, mb = object_mask(real), object_mask(ours)
    both = ma & mb
    d = (ours.sum(axis=2) - real.sum(axis=2)) / 3.0              # + = ours lighter
    k = np.clip(d / full, -1, 1)[..., None]
    lighter, darker = np.array([255, 60, 0]), np.array([0, 70, 255])
    grey = np.array([128, 128, 128])
    pic = np.where(k >= 0, grey + (lighter - grey) * k, grey + (darker - grey) * -k)
    pic[~ma & ~mb] = (40, 40, 40)
    pic[ma & ~mb] = (0, 170, 0)
    pic[mb & ~ma] = (255, 0, 255)
    n = max(int(both.sum()), 1)
    stats = dict(both=int(both.sum()), only_real=int((ma & ~mb).sum()), only_ours=int((mb & ~ma).sum()),
                 mean_abs=float(np.abs(d[both]).sum() / n), mean_signed=float(d[both].sum() / n),
                 off_by_8=float((np.abs(d[both]) > 8).sum() / n))
    return pic.astype(np.uint8), stats, ma | mb


if __name__ == '__main__':
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('dir'); ap.add_argument('--out'); ap.add_argument('--scale', type=int, default=3)
    ap.add_argument('--full', type=float, default=30)
    ap.add_argument('--crop', type=int, default=0, help='centre window in capture pixels; default: the one lineup.png used')
    a = ap.parse_args()
    d = pathlib.Path(a.dir)
    out = pathlib.Path(a.out) if a.out else d / 'pairs'
    out.mkdir(parents=True, exist_ok=True)
    key = (d / 'key.txt').read_text().splitlines()
    reals = [k for k, line in enumerate(key) if line.split(':', 1)[1].strip().startswith('real')]
    if len(key) < 2 or len(reals) != 1:
        raise SystemExit(f'{d}: need one real entry and one of ours (or several of ours, see variants.md); key.txt says {key}')
    r = reals[0]
    views = len(list(d.glob('e0v*.png')))
    print(f'real = entry {r + 1}; score = mean |grey difference| (0-255) where both show the object')
    # the same centre window lineup.py cut for lineup.png: outside it the neighbouring entry can stand in the frame
    c = a.crop or (Image.open(d / 'lineup.png').width // len(key) if (d / 'lineup.png').exists() else 440)
    summary = []
    for o in [k for k in range(len(key)) if k != r]:
        tag = f'e{o}_' if len(key) > 2 else ''
        if len(key) > 2:
            print(f'entry {o + 1}:{key[o].split(":", 1)[1]}')
        scores = []
        for v in range(views):
            real, ours = load(d / f'e{r}v{v}.png'), load(d / f'e{o}v{v}.png')
            h, w = real.shape[:2]
            wx, wy = max((w - c) // 2, 0), max((h - c) // 2, 0)
            real, ours = real[wy:wy + c, wx:wx + c], ours[wy:wy + c, wx:wx + c]
            pic, s, union = compare(real, ours, a.full)
            ys, xs = np.nonzero(union)
            x0, x1 = max(xs.min() - 12, 0), min(xs.max() + 13, real.shape[1])
            y0, y1 = max(ys.min() - 12, 0), min(ys.max() + 13, real.shape[0])
            gap = np.full((y1 - y0, 4, 3), 255, dtype=np.uint8)
            sheet = np.concatenate([real[y0:y1, x0:x1].astype(np.uint8), gap, ours[y0:y1, x0:x1].astype(np.uint8), gap, pic[y0:y1, x0:x1]], axis=1)
            im = Image.fromarray(sheet)
            im.resize((im.width * a.scale, im.height * a.scale), Image.NEAREST).save(out / f'pair_{tag}v{v}.png')
            scores.append(s['mean_abs'])
            print(f'view {v}: score {s["mean_abs"]:5.2f}   ours on average {abs(s["mean_signed"]):.1f} {"lighter" if s["mean_signed"] > 0 else "darker"}; '
                  f'{100 * s["off_by_8"]:.0f}% of pixels differ by more than 8; outline: {s["only_real"]} px only real, {s["only_ours"]} px only ours '
                  f'(of {s["both"]} shared); picture starts at x {wx + x0}, y {wy + y0} of the capture')
        print(f'mean score {sum(scores) / max(views, 1):.2f}   -> {out}')
        summary.append((sum(scores) / max(views, 1), o, scores))
    if len(summary) > 1:
        print('variants, best first:')
        for mean, o, scores in sorted(summary):
            print(f'  {mean:5.2f}  entry {o + 1} ' + ' '.join(f'{v:5.2f}' for v in scores) + f'  {key[o].split(":", 1)[1].strip()}')
    print('note: the score is a difference in grey level, not a ratio. Ratios exaggerate on dark faces: at grey 25 one '
          'lightness step is already 10-15%.')
