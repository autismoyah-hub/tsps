"""Read game colours back from rendered pixels: which (hue, saturation, lit lightness) is this pixel?

Every pixel of a classic capture is an entry of the game's colour table (gamecol.py, the client's own formula),
so the lookup is exact, not a fit. What you get is hue, saturation and the lightness AFTER lighting; the painted
lightness is higher (lit = paint x light / 128; see predict_light.py for the light on a given face).

usage (IMAGE = a capture OUT/eKvR.png, 1280x900; coordinates are pixels in that picture):
  px2col.py IMAGE x,y [x,y ...]             each pixel: every colour that gives exactly this RGB
  px2col.py IMAGE rect X0,Y0,X1,Y1          the colours inside a rectangle: hue/sat, pixel count, lit lightness range
  px2col.py IMAGE col X [Y0 Y1]             runs of colour down one pixel column (layer heights in pixels)
  px2col.py IMAGE row Y [X0 X1]             runs along one pixel row
  add --ground to keep the ground's colour in rect output (dropped by default)

Limits. Several colours can share one RGB, mostly at low lightness (below about 12 the hue is a guess, and
saturation too below about 6). rect/col/row settle this by taking the fewest hue/sat pairs that explain all the
pixels, most common first; a pair marked `?` had rival pairs explaining just as many pixels. Read hue and saturation
from the LIT side of the object and from more than a handful of pixels. A picture that was resized or
anti-aliased (lineup.png's numbers, enlarged pairs) is not exact; such pixels print as `~` with the nearest colour.
"""
import sys, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import numpy as np
from PIL import Image
import gamecol

_rev = None


def candidates(p):
    """All (h, s, l) whose table RGB equals pixel p exactly; [] if p is not a game colour."""
    global _rev
    if _rev is None:
        _rev = {}
        t = gamecol.table()
        for c, k in enumerate((t[:, 0] << 16 | t[:, 1] << 8 | t[:, 2]).tolist()):
            _rev.setdefault(k, []).append(c)
    return [gamecol.split(c) for c in _rev.get(int(p[0]) << 16 | int(p[1]) << 8 | int(p[2]), [])]


def resolve(pixels):
    """Give each pixel one (h, s, l): the fewest hue/sat pairs that explain all pixels, most common first.
    Returns (list of (h, s, l, exact) per pixel, set of hue/sat pairs that won only on a tie)."""
    cand = {}
    for p in set(pixels):
        c = candidates(p)
        cand[p] = (c, True) if c else ([gamecol.nearest(p)[:3]], False)
    count = {}
    for p in pixels:
        count[p] = count.get(p, 0) + 1
    left, chosen, tied = set(cand), {}, set()
    while left:
        votes = {}
        for p in left:
            for hs in {c[:2] for c in cand[p][0]}:
                votes[hs] = votes.get(hs, 0) + count[p]
        best = max(votes.values())
        winners = sorted(hs for hs, n in votes.items() if n == best)
        # among ties prefer a pair already chosen nearby in hue, else the lowest; mark it uncertain
        hs = winners[0]
        if len(winners) > 1:
            tied.add(hs)
        for p in list(left):
            ls = [c[2] for c in cand[p][0] if c[:2] == hs]
            if ls:
                chosen[p] = (hs[0], hs[1], ls[0], cand[p][1]); left.discard(p)
    return [chosen[p] for p in pixels], tied


def ground_of(im):
    """The ground's possible hue/sat pairs: those of the picture's commonest colour."""
    vals, counts = np.unique(im.reshape(-1, 3), axis=0, return_counts=True)
    return {c[:2] for c in candidates(vals[counts.argmax()])}


def runs(pixels):
    """Runs of one hue/sat along a line of pixels: [(hs, first index, last index, [lit lightness...], uncertain)]."""
    res, tied = resolve(pixels)
    out = []
    for k, (h, s, l, exact) in enumerate(res):
        key = (h, s) if exact else ('~', '~')
        if out and out[-1][0] == key:
            out[-1][2] = k; out[-1][3].append(l)
        else:
            out.append([key, k, k, [l], (h, s) in tied])
    return out


if __name__ == '__main__':
    args = [a for a in sys.argv[1:] if a != '--ground']
    if len(args) < 2:
        raise SystemExit(__doc__)
    im = np.asarray(Image.open(args[0]).convert('RGB'))
    H, W = im.shape[:2]
    mode = args[1]
    if mode in ('col', 'row'):
        k = int(args[2])
        a0, a1 = (int(args[3]), int(args[4])) if len(args) > 4 else (0, (H if mode == 'col' else W) - 1)
        line = im[a0:a1 + 1, k] if mode == 'col' else im[k, a0:a1 + 1]
        g = ground_of(im)
        print(f'{mode} {k}, {"y" if mode == "col" else "x"} {a0}..{a1}:   from-to  length  hue/sat  lit lightness first..last (min-max)')
        for key, s, e, ls, unsure in runs([tuple(p) for p in line.tolist()]):
            note = '  ground' if key in g else '  not a game colour (resized picture?)' if key[0] == '~' else '  ? hue/sat uncertain (dark)' if unsure else ''
            print(f'   {a0 + s:4d}-{a0 + e:4d}  {e - s + 1:4d}    {key[0]}/{key[1]}    {ls[0]}..{ls[-1]} ({min(ls)}-{max(ls)}){note}')
    elif mode == 'rect':
        x0, y0, x1, y1 = map(int, args[2].split(','))
        px = [tuple(p) for p in im[y0:y1 + 1, x0:x1 + 1].reshape(-1, 3).tolist()]
        res, tied = resolve(px)
        g = set() if '--ground' in sys.argv else ground_of(im)
        tab = {}
        for h, s, l, exact in res:
            if (h, s) not in g:
                tab.setdefault((h, s, exact), []).append(l)
        n = sum(len(v) for v in tab.values()) or 1
        print(f'rect {x0},{y0}..{x1},{y1}: {len(px)} pixels, {n} not ground')
        print('  hue/sat  pixels  share   lit lightness min..max (median)   commonest lit lightness')
        for (h, s, exact), ls in sorted(tab.items(), key=lambda kv: -len(kv[1])):
            ls = sorted(ls)
            top = sorted(set(ls), key=lambda v: -ls.count(v))[:3]
            print(f'  {h:3d}/{s}  {len(ls):6d}  {100 * len(ls) / n:5.1f}%   {ls[0]:3d}..{ls[-1]:<3d} ({ls[len(ls) // 2]:3d})'
                  f'                  {", ".join(map(str, top))}{"" if exact else "   ~ not exact"}{"   ? uncertain (ties)" if (h, s) in tied else ""}')
    else:
        for a in args[1:]:
            x, y = map(int, a.split(','))
            p = im[y, x]
            c = candidates(p)
            if c:
                print(f'{x},{y}: rgb {tuple(int(v) for v in p)} = ' + '  or  '.join(f'{h}/{s}/{l}' for h, s, l in c)
                      + ('' if len(c) == 1 else f'   ({len(c)} colours share this RGB; use rect on a patch to settle)'))
            else:
                h, s, l, d = gamecol.nearest(p)
                print(f'{x},{y}: rgb {tuple(int(v) for v in p)} ~ {h}/{s}/{l} (nearest, distance {d:.1f}; not a game colour: resized or anti-aliased picture?)')
