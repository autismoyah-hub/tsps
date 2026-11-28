"""shots.py: draw a piece of the REAL map with NPCs standing in it, through the client's own map loader and painter
(client/rust-client/examples/arena_view.rs = asset_arena.rs + `plane N`; see rust/patch_arena_view.py).

  real(out_png, plane, cam, npcs=[(id, x, y, angle)], size=(1280, 900))
       cam = (world_x, world_y, yaw, distance, elevation, target_up): the SAME six numbers tools/scene.py's capture takes,
       the same lens (1000 px at 1280 wide) and the same look_at, so a real arena and one of ours are framed alike.
  cams(cx, cy, span, player)   the three cameras of the study: 'top' (straight down, the whole arena), 'threeq' (from the
       south-west corner, the whole arena) and 'played' (behind the player's tile looking north, a fixed zoomed-out game look).
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, pathlib, subprocess, tempfile, shutil
sys.dont_write_bytecode = True
HERE = pathlib.Path(__file__).resolve().parent
TF = HERE.parents[1]
sys.path.insert(0, str(TF / 'tools'))
import osrscache as oc
from PIL import Image
EXE = _AP.bindir() / 'arena_view.exe'

PLAYED = (2300, 1650, 90)       # distance, elevation, target_up of the played camera (fixed for every arena)


def cams(cx, cy, span, player):
    """cx, cy: arena centre in tiles (fractional); span: its longer side in tiles; player: the tile the player stands on."""
    e = max(1500, span * 128 * 1.32)
    return {'top': (cx, cy, 0, 250, min(11900, e), 0),
            'threeq': (cx, cy, 14336, min(11900, e * 0.86), min(11900, e * 0.70), 60),
            'played': (player[0] + .5, player[1] + .5, 0) + PLAYED}


def real(out, plane, cam, npcs=(), size=(1280, 900), cache=None, extra=()):
    out = pathlib.Path(out); out.parent.mkdir(parents=True, exist_ok=True)
    tmp = pathlib.Path(tempfile.mkdtemp(prefix='arena-'))
    L = [f'size {size[0]} {size[1]}', f'plane {plane}', 'camera ' + ' '.join(str(c) for c in cam), 'pin']
    for n in npcs: L.append(f'npc {n[0]} {n[1]} {n[2]} {n[3] if len(n) > 3 else 0}')
    L += list(extra)
    L.append(f'render 2 {(tmp / "f").as_posix()}')
    (tmp / 's.txt').write_text('\n'.join(L) + '\n', encoding='utf-8')
    r = subprocess.run([str(EXE), str(cache or oc.CACHE), str(tmp / 's.txt')], capture_output=True, text=True)
    if r.returncode:
        raise SystemExit('arena_view failed:\n' + '\n'.join(L) + '\n' + r.stdout[-2000:] + r.stderr[-2000:])
    Image.open(tmp / 'f-0002.bmp').convert('RGB').save(out)
    shutil.rmtree(tmp, ignore_errors=True)
    return out


if __name__ == '__main__':
    # shots.py OUT.png PLANE X Y YAW DIST ELEV UP [npc:id:x:y[:angle] ..]
    a = sys.argv
    n = [tuple(int(v) for v in s.split(':')[1:]) for s in a[9:]]
    print(real(a[1], int(a[2]), tuple(float(v) for v in a[3:9]), n))
