"""livelib.py: shared helpers for the research-live step scripts (paths, running an example, pixel comparison, boards)."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import subprocess, pathlib, sys
from PIL import Image, ImageDraw, ImageFont, ImageChops
import numpy as np

HERE = pathlib.Path(__file__).resolve().parent
ROOT = HERE.parents[1]
EX = _AP.bindir()
SRC = _AP.fix(r'<OSRS_CACHE_239>')
PACKED = str(HERE / 'cache-packed')
LOG = []


def run(exe, *args, cwd=None):
    cmd = [str(EX / exe) if not str(exe).endswith('.exe') or not pathlib.Path(exe).is_absolute() else str(exe)] + [str(a) for a in args]
    if not cmd[0].endswith('.exe'): cmd[0] += '.exe'
    r = subprocess.run(cmd, capture_output=True, text=True, cwd=cwd)
    LOG.append('$ ' + ' '.join(cmd)); LOG.append((r.stdout + r.stderr).rstrip())
    if r.returncode:
        raise SystemExit(f'FAILED: {" ".join(cmd)}\n{r.stdout}{r.stderr}')
    return r.stdout


def diff(a, b):
    """(differing pixels, total pixels, pixels that are not the background in a)."""
    A = np.asarray(Image.open(a).convert('RGB')).astype(int); Bm = np.asarray(Image.open(b).convert('RGB')).astype(int)
    if A.shape != Bm.shape: return (-1, A.shape[0] * A.shape[1], 0)
    d = (A != Bm).any(axis=2)
    bg = A[0, 0]
    return int(d.sum()), int(d.size), int((A != bg).any(axis=2).sum())


def diff_image(a, b, out):
    A = Image.open(a).convert('RGB'); Bm = Image.open(b).convert('RGB')
    d = np.asarray(ImageChops.difference(A, Bm)).any(axis=2)
    im = np.asarray(A).copy() // 3
    im[d] = (255, 0, 255)
    Image.fromarray(im).save(out)


def font(size=15):
    try: return ImageFont.truetype('arial.ttf', size)
    except Exception: return ImageFont.load_default()


def board(out, cells, cols, cell=None, title=None):
    """cells: [(image path or PIL image, caption)]. Tiles them cols wide with captions underneath."""
    ims = [(Image.open(p).convert('RGB') if not isinstance(p, Image.Image) else p.convert('RGB'), c) for p, c in cells]
    if cell:
        for im, _ in ims: im.thumbnail(cell)
    w = max(i.width for i, _ in ims); h = max(i.height for i, _ in ims)
    rows = (len(ims) + cols - 1) // cols
    top = 30 if title else 0
    sheet = Image.new('RGB', (cols * (w + 8) + 8, top + rows * (h + 44) + 8), (24, 24, 28))
    dr = ImageDraw.Draw(sheet)
    if title: dr.text((10, 6), title, fill=(255, 255, 255), font=font(17))
    for k, (im, cap) in enumerate(ims):
        x = 8 + (k % cols) * (w + 8); y = top + 8 + (k // cols) * (h + 44)
        sheet.paste(im, (x, y))
        for n, line in enumerate(cap.split('\n')[:2]):
            dr.text((x + 2, y + h + 3 + n * 18), line, fill=(230, 230, 230), font=font(14))
    pathlib.Path(out).parent.mkdir(parents=True, exist_ok=True)
    sheet.save(out)
    return out


def save_log(path):
    pathlib.Path(path).write_text('\n'.join(LOG) + '\n', encoding='utf-8')
