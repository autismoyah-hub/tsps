"""An animated picture of animations side by side, at the game's own timing, for the owner's review page.

usage: anim_gif.py OUT.gif ROW [ROW...] [--cell WxH] [--zoom N] [--yaw N] [--labels A,B,...] [--step CYCLES]
ROW is what studies/research-animation/apose.py takes: `seq=ID;ENTRY` (a real sequence) or `anim=DIR;ENTRY` (one of
ours as loose files). Each ROW becomes one column of the picture; all play together and loop. One picture frame is
STEP client cycles (default 2 = 40 ms, GIF's practical minimum); a shorter animation holds its last pose until the
longest ends, then all restart after a short pause. Labels are drawn above the columns (default 1, 2, 3 ...).
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, subprocess, pathlib, tempfile
from PIL import Image, ImageDraw

argv = sys.argv[1:]


def opt(name, default=None):
    if name in argv:
        k = argv.index(name); v = argv[k + 1]; del argv[k:k + 2]; return v
    return default


cell = opt('--cell', '260x320'); zoom = opt('--zoom', '300'); yaw = opt('--yaw'); labels = opt('--labels'); step = int(opt('--step', '2'))
out, rows = argv[0], argv[1:]
cw, ch = map(int, cell.split('x'))
APOSE = _AP.fix(r'<ROOT>\studies\research-animation\apose.py')
tmp = pathlib.Path(tempfile.mkdtemp(prefix='animgif-'))
cmd = [sys.executable, '-I', APOSE, str(tmp / 'sheet.png'), *rows, '--time', str(step), '--zoom', zoom, '--cell', cell]
if yaw: cmd += ['--yaw', yaw]
r = subprocess.run(cmd, capture_output=True, text=True)
if r.returncode: sys.exit(r.stdout + r.stderr)
sheet = Image.open(tmp / 'sheet.png').convert('RGB')
n = len(rows); rh = sheet.height // n; steps = sheet.width // cw
names = labels.split(',') if labels else [str(i + 1) for i in range(n)]
frames = []
bg = sheet.getpixel((sheet.width - 2, sheet.height - 2))
last = [None] * n
for t in range(steps):
    f = Image.new('RGB', (cw * n, ch + 22), (22, 27, 34)); d = ImageDraw.Draw(f)
    for k in range(n):
        c = sheet.crop((t * cw, k * rh + (rh - ch), (t + 1) * cw, (k + 1) * rh))
        # an exhausted strip is blank: hold the last drawn pose
        if c.getcolors(4) is not None and len(c.getcolors(4)) <= 2 and last[k] is not None: c = last[k]
        else: last[k] = c
        f.paste(c, (k * cw, 22)); d.text((k * cw + 8, 5), names[k], fill=(235, 235, 235))
    frames.append(f)
frames += [frames[-1]] * max(1, 400 // (20 * step))
frames[0].save(out, save_all=True, append_images=frames[1:], duration=20 * step, loop=0, optimize=False)
print(out, f'{n} columns, {steps} steps of {step} cycles, {frames[0].size}')
