"""Contact sheets of animations, posed by the client's code (asset_anim.exe): how an author sees an animation, and how
a new one is judged beside a real one. One strip per ROW, one cell per frame (or per time step).

usage: apose.py OUT.png ROW [ROW...] [--time STEP] [--zoom N] [--yaw N] [--pitch N] [--cell WxH] [--aim X]
                [--under SEQ:FRAME] [--draw MODE] [--crop L,T,R,B] [--scale K]
ROW = SOURCE;ENTRY[;LABEL]
   SOURCE  seq=ID          a real sequence from the cache
         | anim=DIR        a loose sequence directory (author.py or seqtool.py export)
   ENTRY   what asset_pose/asset_anim accept: none, item:ID, model:PATH@weapon, outfits with +, npc:ID
Without --time each strip shows every frame once, captioned `f3 4c` (frame 3, held 4 client cycles of 20 ms).
With --time STEP every strip is sampled at 0, STEP, 2*STEP ... cycles, so strips of different animations line up
in TIME: a held frame repeats, a 1-cycle frame may be skipped (the caption says which frame is showing).
--crop cuts each cell after drawing (pixels, in the cell); --scale K enlarges the finished sheet by nearest pixel.
Writes OUT.png and OUT.key.txt.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, subprocess, pathlib
from PIL import Image, ImageDraw, ImageFont
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import animlib as A
EXE = _AP.fix(r'<CLIENT_BIN>\asset_anim.exe')
argv = sys.argv


def opt(name, default=None):
    return argv[argv.index(name) + 1] if name in argv else default


def main():
    out = pathlib.Path(argv[1]); out.parent.mkdir(parents=True, exist_ok=True)
    rows = []
    i = 2
    while i < len(argv) and not argv[i].startswith('--'):
        rows.append(argv[i]); i += 1
    step = int(opt('--time')) if opt('--time') else None
    zoom = int(opt('--zoom', '520'))
    cell = opt('--cell') or ('300x400' if zoom >= 400 else '600x800')
    cw, ch = map(int, cell.split('x'))
    crop = tuple(map(int, opt('--crop').split(','))) if opt('--crop') else (0, 0, cw, ch)
    extra = [f'{k}={opt("--" + k)}' for k in ('zoom', 'yaw', 'pitch', 'aim', 'under', 'draw') if opt('--' + k)] + [f'cell={cell}']
    strips, key = [], []
    for row in rows:
        parts = row.split(';')
        source, entry = parts[0], parts[1]
        label = parts[2] if len(parts) > 2 else source
        if source.startswith('seq='):
            s = A.sequence(int(source[4:]))
        else:
            s = A.decode_sequence((pathlib.Path(source[5:]) / 'seq.dat').read_bytes())
        L = s['lengths']
        if step:
            total = sum(L); want = []
            t = 0
            while t < total:
                acc = 0
                for k, n in enumerate(L):
                    if t < acc + n: want.append((k, t)); break
                    acc += n
                t += step
        else:
            want = [(k, None) for k in range(len(L))]
            if opt('--frames'):      # --frames 0,3,5: only these frames (applies to every row)
                want = [(int(k), None) for k in opt('--frames').split(',') if int(k) < len(L)]
        uniq = sorted({k for k, _ in want})
        bmp = out.with_suffix(f'.{len(strips)}.bmp')
        r = subprocess.run([EXE, str(A.CACHE), str(bmp), source, 'frames=' + ','.join(map(str, uniq)), *extra, entry], capture_output=True, text=True)
        if r.returncode:
            raise SystemExit(r.stdout + r.stderr)
        for line in r.stdout.splitlines():
            if 'WARNING' in line: print(line)
        im = Image.open(bmp).convert('RGB'); bmp.unlink()
        cells = {k: im.crop((0, n * ch, cw, (n + 1) * ch)).crop(crop) for n, k in enumerate(uniq)}
        strips.append((label, [(cells[k], f'f{k} {L[k]}c' if t is None else f't{t} f{k}') for k, t in want]))
        key.append(f'{label}: {source} {entry} lengths {L} total {sum(L)} cycles')
    w, h = crop[2] - crop[0], crop[3] - crop[1]
    n = max(len(c) for _, c in strips)
    head = 22
    sheet = Image.new('RGB', (n * w, len(strips) * (h + head)), (48, 43, 37))
    d = ImageDraw.Draw(sheet)
    try: font = ImageFont.truetype('segoeuib.ttf', 15); small = ImageFont.truetype('segoeui.ttf', 13)
    except Exception: font = small = ImageFont.load_default()
    for r, (label, cells) in enumerate(strips):
        y = r * (h + head)
        d.rectangle([0, y, sheet.width, y + head - 1], fill=(30, 27, 23))
        d.text((6, y + 2), label, fill=(240, 232, 200), font=font)
        for c, (im, cap) in enumerate(cells):
            sheet.paste(im, (c * w, y + head))
            d.text((c * w + 5, y + head + h - 18), cap, fill=(200, 200, 160), font=small)
    k = int(opt('--scale', '1'))
    if k > 1: sheet = sheet.resize((sheet.width * k, sheet.height * k), Image.NEAREST)
    sheet.save(out)
    out.with_suffix('.key.txt').write_text(' '.join(argv[1:]) + '\n' + '\n'.join(key) + '\n')
    print(out, sheet.size)
    for k_ in key: print(' ', k_)


if __name__ == '__main__':
    main()
