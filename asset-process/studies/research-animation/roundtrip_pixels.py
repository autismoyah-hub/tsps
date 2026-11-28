"""Round-trip proof in pixels. For each sequence: (1) asset_pose.exe poses the figure from the CACHE with the client's
Animations::apply_actor; (2) asset_anim.exe poses it from the RE-ENCODED loose files in rt/SEQ. The two pictures are
compared cell by cell (one cell per entry and frame).

usage: roundtrip_pixels.py SEQ [SEQ...] [--under SEQ:FRAME] [--corrupt]
--corrupt adds 1 to one rotation value of frame 1 of the loose copy first (a negative control: the comparison must
then report differing pixels), and restores the file afterwards.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, subprocess, pathlib
from PIL import Image, ImageChops
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import animlib as A
HERE = pathlib.Path(__file__).parent
EX = pathlib.Path(_AP.fix(r'<CLIENT_BIN>'))
CACHE = str(A.CACHE)
OUT = pathlib.Path(_AP.fix(r'<ROOT>\render\research-animation-roundtrip')); OUT.mkdir(parents=True, exist_ok=True)
ENTRIES = ['none', 'item:1323', _AP.fix(r'model:<ROOT>/builds/meridian-sword/meridian-sword.bin@weapon'),
           'item:1127+item:1079+item:1163']
argv = sys.argv[1:]
under = argv[argv.index('--under') + 1] if '--under' in argv else None
corrupt = '--corrupt' in argv
seqs = [int(x) for x in argv if x.isdigit()]
total_cells = total_diff = 0
for sid in seqs:
    s, bid, base, frames = A.sequence_frames(sid)
    d = HERE / 'rt' / str(sid)
    restore = None
    if corrupt:
        fid = s['frames'][1]; p = d / f'frame_{fid >> 16}_{fid & 65535}.dat'
        restore = (p, p.read_bytes())
        f = A.decode_frame(restore[1], base)
        g = next(g for g in sorted(f['groups']) if base['types'][g] == 2)
        f['groups'][g][0] += 1
        p.write_bytes(A.encode_frame(dict(base=f['base'], groups=f['groups']), base))
        print(f'negative control: frame 1 group {g} x-rotation raised by 1 in {p.name}')
    fl = ','.join(str(i) for i in range(len(frames)))
    extra = [f'under={under}'] if under else []
    tag = f'{sid}' + (f'-under-{under.replace(":", "_")}' if under else '') + ('-corrupt' if corrupt else '')
    a, b = OUT / f'{tag}-cache.bmp', OUT / f'{tag}-files.bmp'
    try:
        r1 = subprocess.run([str(_AP.exe('asset_pose.exe', EX)), CACHE, str(a), f'seq={sid}', f'frames={fl}', *extra, *ENTRIES], capture_output=True, text=True)
        r2 = subprocess.run([str(_AP.exe('asset_anim.exe', EX)), CACHE, str(b), f'anim={d}', f'frames={fl}', *extra, *ENTRIES], capture_output=True, text=True)
    finally:
        if restore: restore[0].write_bytes(restore[1])
    if r1.returncode or r2.returncode:
        raise SystemExit(r1.stdout + r1.stderr + r2.stdout + r2.stderr)
    ia, ib = Image.open(a).convert('RGB'), Image.open(b).convert('RGB')
    assert ia.size == ib.size
    cw, ch = ia.width // len(ENTRIES), ia.height // len(frames)
    bg = ia.getpixel((0, 0))
    worst = 0
    for row in range(len(frames)):
        for col in range(len(ENTRIES)):
            box = (col * cw, row * ch, (col + 1) * cw, (row + 1) * ch)
            ca, cb = ia.crop(box), ib.crop(box)
            diff = sum(1 for p, q in zip(ca.getdata(), cb.getdata()) if p != q)
            drawn = sum(1 for p in ca.getdata() if p != bg)
            assert drawn > 2000, 'empty cell'
            total_cells += 1; total_diff += diff; worst = max(worst, diff)
            if diff: print(f'  seq {sid} frame {row} entry {col + 1}: {diff} differing pixels of {cw * ch}')
    ia.save(a.with_suffix('.png')); ib.save(b.with_suffix('.png')); a.unlink(); b.unlink()
    print(f'seq {sid}{" under " + under if under else ""}: {len(frames)} frames x {len(ENTRIES)} entries, cell {cw}x{ch}; worst cell {worst} differing pixels; '
          f'pictures {a.with_suffix(".png").name}, {b.with_suffix(".png").name}; bytes identical: {a.with_suffix(".png").read_bytes() == b.with_suffix(".png").read_bytes()}')
print(f'TOTAL: {total_cells} cells, {total_diff} differing pixels')
