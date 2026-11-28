"""cape_render.py: capes on the standard figures with armour, our legs and a staff, four sides, and a layering count.

usage: cape_render.py     (writes <ROOT>/render/research-items-capes/*.png and prints the counts)
For each gender and yaw (192 front quarter, 704 and 1344 sides, 1024 back) one sheet: walk 819 frames bind, 1, 5.
Layering count: the same sheet drawn with --draw actor (the game's painting of a player) and --draw depth (true
per-pixel depth); pixels that differ, per column, minus the same count for the column WITHOUT the cape, is what the
cape adds to the figure's painting-order errors.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, subprocess, pathlib
from PIL import Image, ImageChops
R = pathlib.Path(_AP.fix('<ROOT>/render/research-items-capes')); R.mkdir(parents=True, exist_ok=True)
PY = [_AP.fix(r'<PYTHON>'), '-I', _AP.fix('<ROOT>/studies/research-items/fpose.py')]
LEGS = _AP.fix('<ROOT>/builds/meridian-legs/meridian-legs-labelled.bin')
STAFF = _AP.fix('<ROOT>/builds/meridian-staff/meridian-staff.bin')
PLATE = 'item:1127+item:1079'
# pairs: (without cape, with cape)
PAIRS = [('none', 'item:1007'), (PLATE, PLATE + '+item:1007'), (PLATE, PLATE + '+item:6570'), (PLATE, PLATE + '+item:9747'),
         (f'model:{STAFF}', f'item:1007+model:{STAFF}'), ('item:1127+item:1079+item:3204', PLATE + '+item:3204+item:1007')]
MALE_ONLY = [(f'model:{LEGS}@legs', f'model:{LEGS}@legs+item:1007'), (f'model:{LEGS}@legs', f'model:{LEGS}@legs+item:9747')]
FRAMES = 'bind,1,5'


def sheet(tag, entries, gender, yaw, draw, seq='819'):
    out = R / f'{tag}.png'
    r = subprocess.run(PY + [str(out), ','.join(entries), '--gender', str(gender), '--yaw', str(yaw), '--seq', seq, '--frames', FRAMES, '--draw', draw, '--jaw', '0'],
                       capture_output=True, text=True)
    if r.returncode: raise SystemExit(r.stdout + r.stderr)
    return Image.open(out).convert('RGB')


def count(a, b, k, n, rows=3):
    w = a.width // n; h = a.height // rows; out = []
    for r in range(rows):
        box = (k * w, r * h + 40, (k + 1) * w, (r + 1) * h - 30)      # leave out the column number and frame caption
        d = ImageChops.difference(a.crop(box), b.crop(box)).convert('L').point(lambda v: 255 if v else 0)
        out.append(sum(d.histogram()[1:]))
    return out


for gender, gname in ((0, 'male'), (1, 'female')):
    pairs = PAIRS + (MALE_ONLY if gender == 0 else [])
    with_cape = [p[1] for p in pairs]; without = [p[0] for p in pairs]
    for yaw in (192, 704, 1024, 1344):
        a = sheet(f'{gname}-yaw{yaw}-actor', with_cape, gender, yaw, 'actor')
        d = sheet(f'{gname}-yaw{yaw}-depth', with_cape, gender, yaw, 'depth')
        a0 = sheet(f'_base-{gname}-yaw{yaw}-actor', without, gender, yaw, 'actor')
        d0 = sheet(f'_base-{gname}-yaw{yaw}-depth', without, gender, yaw, 'depth')
        print(f'{gname} yaw {yaw}: pixels where the game\'s painting differs from true depth, frames {FRAMES} (with cape | same outfit without cape)')
        for k, (p0, p1) in enumerate(pairs):
            short = p1.replace(LEGS, 'MERIDIAN-LEGS').replace(STAFF, 'MERIDIAN-STAFF')
            print(f'   col {k + 1} {short}: {count(a, d, k, len(pairs))} | {count(a0, d0, k, len(pairs))}')
for f in R.glob('_base-*'): f.unlink()
