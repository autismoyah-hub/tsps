"""Proof that a packed cache plays the new sequence ids through the client's ordinary loader.

usage: packcheck.py PACKED_CACHE LOOSE_DIR:SEQ_ID:ENTRY [...]
For each job the UNMODIFIED asset_pose.exe (Animations::load -> Sequence::load_all, DiskCache::group with CRC and
version checks, Animations::apply_actor) poses ENTRY from the packed cache by sequence id, every frame, and the
picture is compared with asset_anim.exe posing the same loose directory against the ORIGINAL cache.
Also poses real sequences 819 and 390 from both caches (the packed copy must not have disturbed them).
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, subprocess, pathlib
from PIL import Image, ImageChops
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import animlib as A
EX = pathlib.Path(_AP.fix(r'<CLIENT_BIN>'))
OUT = pathlib.Path(_AP.fix(r'<ROOT>\render\research-animation-packed')); OUT.mkdir(parents=True, exist_ok=True)
packed = sys.argv[1]


def run(exe, cache, bmp, *args):
    r = subprocess.run([str(EX / exe), str(cache), str(bmp), *args], capture_output=True, text=True)
    if r.returncode: raise SystemExit(f'{exe} failed: ' + r.stdout[-600:] + r.stderr[-600:])
    im = Image.open(bmp).convert('RGB'); pathlib.Path(bmp).unlink()
    return im, r.stdout


def differing(a, b):
    assert a.size == b.size
    return sum(1 for p in ImageChops.difference(a, b).getdata() if p != (0, 0, 0))


total = 0
for job in sys.argv[2:]:
    d, sid, entry = job.split(':', 2)
    s = A.decode_sequence((pathlib.Path(d) / 'seq.dat').read_bytes())
    fl = 'frames=' + ','.join(map(str, range(len(s['frames']))))
    draw = ['draw=auto'] if entry.startswith('npc:') else []
    if entry.startswith('npc:'):
        # asset_pose has no npc entry: use asset_anim's cache path (seq=ID) on the packed cache instead
        a, out = run('asset_anim.exe', packed, OUT / '_a.bmp', f'seq={sid}', fl, entry)
        how = 'asset_anim.exe seq=ID on the packed cache (Sequence::load_all + DiskCache::group)'
    else:
        a, out = run('asset_pose.exe', packed, OUT / '_a.bmp', f'seq={sid}', fl, entry)
        how = 'unmodified asset_pose.exe on the packed cache'
    b, _ = run('asset_anim.exe', A.CACHE, OUT / '_b.bmp', f'anim={d}', fl, entry)
    n = differing(a, b); total += n
    a.save(OUT / f'packed-{sid}.png')
    print(f'seq {sid} ({d}), {len(s["frames"])} frames, {entry[:40]}: {how} vs loose files on the original cache: {n} differing pixels of {a.width * a.height}')
for sid in (819, 390):
    fl = 'frames=' + ','.join(map(str, range(len(A.sequence(sid)['frames']))))
    a, _ = run('asset_pose.exe', packed, OUT / '_a.bmp', f'seq={sid}', fl, 'none', 'item:1323')
    b, _ = run('asset_pose.exe', A.CACHE, OUT / '_b.bmp', f'seq={sid}', fl, 'none', 'item:1323')
    n = differing(a, b); total += n
    print(f'real seq {sid}: packed cache vs original cache: {n} differing pixels of {a.width * a.height}')
r = subprocess.run([str(_AP.exe('asset_pose.exe', EX)), packed, 'info'] + [j.split(':')[1] for j in sys.argv[2:]], capture_output=True, text=True)
print(r.stdout.strip()[:1500])
print('TOTAL differing pixels:', total)
