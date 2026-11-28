"""Render an arena script at native fixed-client scale, with optional extra samples."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import argparse
import os
from pathlib import Path
import subprocess
import sys
from tempfile import TemporaryDirectory
from PIL import Image

sys.dont_write_bytecode = True
sys.path.insert(0, str(Path(__file__).resolve().parent))
import quality as Q

ARENA = Path(Q.L.cmesh.EXE).with_name("asset_arena.exe")

def capture(lines, scratch, exe=ARENA, cache=Q.L.cmesh.CACHE, cycle=0):
    script = scratch / "stage.txt"
    prefix = scratch / "frame"
    script.write_text("\n".join([*lines, f"render {cycle} {prefix.as_posix()}"]) + "\n")
    subprocess.run([exe, cache, script], capture_output=True, text=True, check=True)
    with Image.open(scratch / f"frame-{cycle:04}.bmp") as im:
        return im.convert("RGB")

def scene(lines, scratch, tile, yaw, preset, factor, cache, cycle=0):
    body = [s for s in lines if s.split() and s.split()[0] not in {"size","focal","camera","render"}]
    settings = Q.arena_settings(preset, factor, tile, yaw)
    return Q.magnify(capture(settings + body, scratch, cache=cache, cycle=cycle), preset, factor)

def authored(content, scratch, tile, yaw, preset, factor, cache):
    size, lens, camera = Q.arena_settings(preset, factor, tile, yaw)
    _, w, h = size.split()
    env = {**os.environ, "REVIEW_W":w, "REVIEW_H":h, "REVIEW_FOCAL":lens.split()[1]}
    dest = scratch / "snapshot"
    subprocess.run([_AP.exe('quality_scene.exe', Q.HERE),content,cache,*camera.split()[1:],
                    "classic","auto",dest],env=env,check=True,capture_output=True)
    with Image.open(dest/"view.bmp") as im:
        return Q.magnify(im.convert("RGB"),preset,factor)

if __name__ == "__main__":
    p = argparse.ArgumentParser(description=__doc__)
    p.add_argument("script", type=Path)
    p.add_argument("out", type=Path)
    p.add_argument("--tile", type=float, nargs=2, required=True)
    p.add_argument("--yaw", type=int, default=0)
    p.add_argument("--cycle", type=int, default=0)
    p.add_argument("--authored", action="store_true", help="input is a scene.py content.txt")
    p.add_argument("--cache", default=Q.L.cmesh.CACHE)
    p.add_argument("--preset", choices=("faithful","magnified","hires"), default="faithful")
    p.add_argument("--factor", type=int, default=2)
    a = p.parse_args()
    if a.out.exists():
        raise FileExistsError(a.out)
    with TemporaryDirectory(dir=Q.HERE) as tmp:
        if a.authored:
            if a.cycle:
                p.error("Authored snapshots use the original snapshot example's fixed tick")
            im = authored(a.script,Path(tmp),a.tile,a.yaw,a.preset,a.factor,a.cache)
        else:
            im = scene(a.script.read_text().splitlines(),Path(tmp),a.tile,a.yaw,a.preset,a.factor,a.cache,a.cycle)
        im.save(a.out, format="PNG")
