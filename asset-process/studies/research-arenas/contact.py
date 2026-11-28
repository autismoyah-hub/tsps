"""contact.py: boards of the real arenas.

  contact.py OUT.png COLS THUMB_W SUFFIX KEY [KEY ..]   thumbnails of render/research-arenas/KEY-SUFFIX.png on one sheet
  contact.py boards                                     writes boards/board-real-{played,top,check,threeq}-N.jpg: every
                                                        measured arena, nine a sheet, labelled with its name and size
SUFFIX: played (the played camera, with the boss), top (straight down, with the boss), threeq, check (straight down with
the measured floor outlined in yellow, the boss's tiles in cyan, the player's tile in magenta), top-empty.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, json, pathlib
from PIL import Image, ImageDraw
HERE = pathlib.Path(__file__).resolve().parent
D = pathlib.Path(_AP.fix('<ROOT>/render/research-arenas'))


def sheet(out, cols, tw, suf, keys, labels=None, quality=90):
    ims = []
    for k in keys:
        im = Image.open(D / f'{k}-{suf}.png').convert('RGB'); im.thumbnail((tw, tw)); ims.append((k, im))
    h = max(i.size[1] for _, i in ims) + 16; rows = (len(ims) + cols - 1) // cols
    S = Image.new('RGB', (cols * tw, rows * h), (20, 20, 24)); dr = ImageDraw.Draw(S)
    for n, (k, im) in enumerate(ims):
        x, y = (n % cols) * tw, (n // cols) * h
        S.paste(im, (x, y)); dr.text((x + 4, y + h - 14), (labels or {}).get(k, k), fill='white')
    S.save(out, quality=quality) if str(out).endswith('.jpg') else S.save(out)
    print(out, S.size)


if __name__ == '__main__':
    if sys.argv[1] == 'boards':
        A = json.loads((HERE / 'data' / 'arenas.json').read_text())
        lab = {k: f"{m['name']}: {m['area']} tiles, {m['w']} x {m['h']}, {m['outline']}; boss {m['boss_side']} tiles, {m['boss_place']}" for k, m in A.items()}
        keys = list(A); (HERE / 'boards').mkdir(exist_ok=True)
        for suf in ('played', 'top', 'check', 'threeq'):
            for n in range(0, len(keys), 9):
                sheet(HERE / 'boards' / f'board-real-{suf}-{n // 9 + 1}.jpg', 3, 640, suf, keys[n:n + 9], lab)
    else:
        sheet(sys.argv[1], int(sys.argv[2]), int(sys.argv[3]), sys.argv[4], sys.argv[5:])
