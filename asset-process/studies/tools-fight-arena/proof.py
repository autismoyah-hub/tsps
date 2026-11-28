"""Compare the empty court with both renderers at the board's recorded top pose."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import json
import subprocess
import sys
import tempfile
from pathlib import Path

import numpy as np
from PIL import Image

sys.dont_write_bytecode = True
D = Path(__file__).resolve().parent
sys.path.insert(0,str(D))
from arena_pack import WARDEN, pack, pose
import fx

f = json.loads((D/'fight-arena.json').read_text())
original = json.loads((WARDEN/'fight.json').read_text())
assert all(f[k] == original[k] for k in ('attacks','timeline','phase'))
base = pack.A.Cache(WARDEN/'cache-own')
packed = pack.A.Cache(f['cache'])
for sid in (20271,20272,20273):
    assert base.file(2,12,sid) == packed.file(2,12,sid)
board = Path(f['board'])
arena = board.parents[1]
out = Path(f['output'])
out.mkdir(exist_ok=True)
view = 'top-empty'
scene = board / ('scene-' + view)
cam = pose(board,view)
cam[2] = int(cam[2])
with tempfile.TemporaryDirectory(dir=out) as tmp:
    tmp = Path(tmp)
    subprocess.run([_AP.exe('authored_scene_snapshot.exe', fx.EXE),scene/'content.txt',arena/'effect-cache',
                    *map(str,cam),'classic','auto',tmp/'room'],check=True)
    room = Image.open(tmp/'room/view.bmp').convert('RGB')
    room.save(out/'room-proof.png')
    assert np.array_equal(np.asarray(room),np.asarray(Image.open(board/'top-empty.png').convert('RGB')))
    rx,ry = f['stage']['region']
    cam[0] += 64*rx
    cam[1] += 64*ry
    prefix = (tmp/'world').as_posix()
    log = fx.run([f'size {room.width} {room.height}', 'camera '+' '.join(map(str,cam)), 'pin', f'render 1 {prefix}'],f['cache'])
    print(log[:900])
    world = fx.frames(prefix,[1])[0]
    world.save(out/'map-proof.png')
    diff = np.any(np.asarray(room)!=np.asarray(world),axis=2)
    Image.fromarray(diff).save(out/'difference.png')
    mask = Image.fromarray(diff)
    result = dict(differing_pixels=int(diff.sum()), pixels=diff.size, bounds=mask.getbbox(),
                  size=room.size, room_pose=pose(board,view), map_pose=cam, cycle=1)
    (out/'pixel-proof.json').write_text(json.dumps(result,indent=2))
    print(f'{diff.sum()} / {diff.size} pixels differ')

