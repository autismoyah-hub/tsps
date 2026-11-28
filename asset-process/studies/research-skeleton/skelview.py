"""skelview.py: pictures of an NPC (real, or ours from files) in sequences that may be loose files with their own frame
base. Wraps examples/asset_skel.rs. Contact sheets, time-lined sheets, moving pictures, pixel comparison.

usage: skelview.py sheet OUT.png ROW [ROW...] [--time STEP] [--frames A,B,..] [--zoom N] [--yaw N] [--pitch N]
                   [--cell WxH] [--aim H] [--scale K] [--cache DIR] [--max N]
       skelview.py gif OUT.gif ROW [ROW...] [--step CYCLES] [--loops N] (same options; one COLUMN per row, played
                   together at the game's timing: one picture frame = STEP client cycles of 20 ms, default 2)
       skelview.py motion ROW [ROW...] [--step 2]   how much of the picture changes every STEP cycles: pops, dead time
       skelview.py track ENTRY SOURCE LABELS       centroids of labels per frame (sliding / contact checks)
ROW = ENTRY;SOURCE[;LABEL]
   ENTRY   npc:ID  |  def:PATH.npcdef (ours: models=FILES, scale=, ambient=, contrast=)
   SOURCE  seq=ID  |  anim=DIR (seq.dat, frame_G_F.dat, base_B.dat)  |  bind
Captions: `f3 4c` = frame 3 held 4 client cycles; with --time, `t12 f3`.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, subprocess, pathlib, tempfile, re
from PIL import Image, ImageDraw, ImageFont
import numpy as np
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE.parent / 'research-animation'))
import animlib as A
EXE = _AP.fix(r'<CLIENT_BIN>\asset_skel.exe')
BG = (0x30, 0x2b, 0x25)


def lengths_of(source, cache=None):
    if source == 'bind': return [1]
    if source.startswith('seq='):
        return A.sequence(int(source[4:]), A.Cache(cache) if cache else None)['lengths']
    return A.decode_sequence((pathlib.Path(source[5:]) / 'seq.dat').read_bytes())['lengths']


def strip(entry, source, frames=None, zoom=520, yaw=192, pitch=0, cell=(300, 400), aim=None, cache=None, track=None, draw=None):
    """-> ([PIL cell per requested frame], stdout). frames None = all."""
    tmp = pathlib.Path(tempfile.mkdtemp(prefix='skelview-')) / 's.bmp'
    cmd = [EXE, str(cache or A.CACHE), str(tmp), f'zoom={zoom}', f'yaw={yaw}', f'pitch={pitch}', f'cell={cell[0]}x{cell[1]}']
    if aim is not None: cmd.append(f'aim={aim}')
    if draw: cmd.append(f'draw={draw}')
    if track: cmd.append('track=' + ','.join(map(str, track)))
    if source != 'bind':
        cmd.append(source)
        if frames is not None: cmd.append('frames=' + ','.join(map(str, frames)))
    cmd.append(entry)
    r = subprocess.run(cmd, capture_output=True, text=True)
    if r.returncode: raise SystemExit(' '.join(cmd) + '\n' + r.stdout + r.stderr)
    im = Image.open(tmp).convert('RGB'); im.load(); tmp.unlink()
    n = im.width // cell[0]
    return [im.crop((k * cell[0], 0, (k + 1) * cell[0], cell[1])) for k in range(n)], r.stdout


def diff(a, b):
    x = np.asarray(a).astype(int); y = np.asarray(b).astype(int)
    d = (x != y).any(axis=2)
    fig = (x != np.array(BG)).any(axis=2)
    return int(d.sum()), int(d.size), int(fig.sum())


def fonts():
    try: return ImageFont.truetype('segoeuib.ttf', 15), ImageFont.truetype('segoeui.ttf', 13)
    except Exception: return ImageFont.load_default(), ImageFont.load_default()


def timeline(L, step):
    want = []; t = 0; total = sum(L)
    while t < total:
        acc = 0
        for k, n in enumerate(L):
            if t < acc + n: want.append((k, t)); break
            acc += n
        t += step
    return want


def parse(argv):
    o = {}
    rows = []
    i = 0
    while i < len(argv):
        if argv[i].startswith('--'): o[argv[i][2:]] = argv[i + 1]; i += 2
        else: rows.append(argv[i]); i += 1
    return rows, o


def rows_cells(rows, o, time=None):
    zoom = int(o.get('zoom', 520)); yaw = int(o.get('yaw', 192)); pitch = int(o.get('pitch', 0))
    cw, ch = map(int, o.get('cell', '300x400').split('x'))
    aim = int(o['aim']) if 'aim' in o else None
    out = []
    for row in rows:
        p = row.split(';')
        entry, source = p[0], p[1]
        label = p[2] if len(p) > 2 else f'{entry} {source}'
        L = lengths_of(source, o.get('cache'))
        if time: want = timeline([min(x, 40) for x in L], time)      # a death's last frame may be held 20000 cycles
        elif 'frames' in o: want = [(int(k), None) for k in o['frames'].split(',') if int(k) < len(L)]
        else: want = [(k, None) for k in range(len(L))]
        if 'max' in o and not time and len(want) > int(o['max']):
            n = int(o['max']); want = [want[round(i * (len(want) - 1) / (n - 1))] for i in range(n)]
        uniq = sorted({k for k, _ in want})
        cells, _ = strip(entry, source, uniq, zoom, yaw, pitch, (cw, ch), aim, o.get('cache'))
        by = dict(zip(uniq, cells))
        out.append((label, [(by[k], f'f{k} {L[k]}c' if t is None else f't{t} f{k}') for k, t in want], L))
    return out, (cw, ch)


def sheet(out, rows, o):
    data, (cw, ch) = rows_cells(rows, o, int(o['time']) if 'time' in o else None)
    font, small = fonts()
    n = max(len(c) for _, c, _ in data); head = 22
    im = Image.new('RGB', (n * cw, len(data) * (ch + head)), (48, 43, 37)); d = ImageDraw.Draw(im)
    key = []
    for r, (label, cells, L) in enumerate(data):
        y = r * (ch + head)
        d.rectangle([0, y, im.width, y + head - 1], fill=(30, 27, 23))
        d.text((6, y + 2), f'{label}   holds {L} = {sum(L)} cycles', fill=(240, 232, 200), font=font)
        for c, (cellim, cap) in enumerate(cells):
            im.paste(cellim, (c * cw, y + head)); d.text((c * cw + 5, y + head + ch - 18), cap, fill=(200, 200, 160), font=small)
        key.append(f'{label}: lengths {L} total {sum(L)}')
    k = int(o.get('scale', 1))
    if k > 1: im = im.resize((im.width * k, im.height * k), Image.NEAREST)
    pathlib.Path(out).parent.mkdir(parents=True, exist_ok=True)
    im.save(out); pathlib.Path(out).with_suffix('.key.txt').write_text(' '.join(sys.argv[1:]) + '\n' + '\n'.join(key) + '\n')
    print(out, im.size)
    for k_ in key: print('  ', k_)


def gif(out, rows, o):
    step = int(o.get('step', 2)); loops = int(o.get('loops', 1))
    data, (cw, ch) = rows_cells(rows, o, step)
    font, small = fonts()
    n = max(len(c) for _, c, _ in data) * loops
    frames = []
    for t in range(n):
        f = Image.new('RGB', (cw * len(data), ch + 22), (22, 27, 34)); d = ImageDraw.Draw(f)
        for k, (label, cells, L) in enumerate(data):
            # a looping row repeats; every row loops on its own length so idles keep cycling
            c = cells[t % len(cells)][0]
            f.paste(c, (k * cw, 22)); d.text((k * cw + 6, 3), label, fill=(235, 235, 235), font=small)
        frames.append(f)
    pathlib.Path(out).parent.mkdir(parents=True, exist_ok=True)
    frames[0].save(out, save_all=True, append_images=frames[1:], duration=20 * step, loop=0, optimize=False)
    print(out, f'{len(data)} columns, {n} steps of {step} cycles, {frames[0].size}')


def track(entry, source, labels, cache=None):
    cells, text = strip(entry, source, None, cell=(40, 40), track=labels, cache=cache)
    out = {}
    boxes = []
    for line in text.splitlines():
        m = re.match(r'track cell (\d+) label (\d+) n \d+ x ([-\d.]+) h ([-\d.]+) z ([-\d.]+)', line)
        if m: out.setdefault(int(m.group(2)), {})[int(m.group(1))] = (float(m.group(3)), float(m.group(4)), float(m.group(5)))
        m = re.match(r'cell (\d+): .*box min \[(-?\d+), (-?\d+), (-?\d+)\] max \[(-?\d+), (-?\d+), (-?\d+)\]', line)
        if m: boxes.append(tuple(int(x) for x in m.groups()[1:]))
    return out, boxes


def motion(rows, o):
    """per row: pixels that change between successive samples STEP cycles apart, as a share of the figure's pixels.
    A spike is a pop; a run of zeros is dead time; a flat line is even timing."""
    step = int(o.get('step', 2))
    data, (cw, ch) = rows_cells(rows, o, step)
    for label, cells, L in data:
        ims = [np.asarray(c[0]).astype(int) for c in cells]
        fig = max(1, int(np.mean([(x != np.array(BG)).any(axis=2).sum() for x in ims])))
        ch_ = [int((a_ != b_).any(axis=2).sum()) * 100 // fig for a_, b_ in zip(ims, ims[1:] + ims[:1] if o.get('loop') else ims[1:])]
        nz = [c for c in ch_ if c]
        print(f'{label}: holds {L}; figure {fig} px; change per {step} cycles, % of the figure: ' + ' '.join(map(str, ch_)))
        print(f'    peak {max(ch_)}%, median of the moving steps {sorted(nz)[len(nz) // 2] if nz else 0}%, still steps {len(ch_) - len(nz)} of {len(ch_)}, peak / median {max(ch_) / max(1, sorted(nz)[len(nz) // 2] if nz else 1):.1f}')


if __name__ == '__main__':
    cmd = sys.argv[1]
    if cmd == 'motion':
        rows, o = parse(sys.argv[2:]); motion(rows, o); raise SystemExit
    if cmd == 'sheet':
        rows, o = parse(sys.argv[3:]); sheet(sys.argv[2], rows, o)
    elif cmd == 'gif':
        rows, o = parse(sys.argv[3:]); gif(sys.argv[2], rows, o)
    elif cmd == 'track':
        t, boxes = track(sys.argv[2], sys.argv[3], [int(x) for x in sys.argv[4].split(',')])
        for l, d in t.items():
            print(f'label {l}: ' + ' | '.join(f'{k}:({v[0]:.0f},{v[1]:.0f},{v[2]:.0f})' for k, v in sorted(d.items())))
        for k, b in enumerate(boxes): print(f'frame {k}: box x {b[0]}..{b[3]} h {-b[4]}..{-b[1]} z {b[2]}..{b[5]}')
