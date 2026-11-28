"""Compare the renderers on plane 0, and isolate each drawn element in the repair film."""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
from pathlib import Path
from tempfile import TemporaryDirectory
import sys
import json
sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import play_real as play
import numpy as np

fx = play.FX
fx.tempfile.tempdir = str(HERE)
stock = fx.EXE_FX
local = _AP.exe('arena_film.exe', HERE)

def capture(exe, lines, times):
    with TemporaryDirectory(dir=HERE) as tmp:
        root = Path(tmp).as_posix()+"/f"
        fx.EXE_FX = exe
        fx.run(lines+[f"render {','.join(map(str,times))} {root}"], fx.SRC)
        return [np.asarray(im) for im in fx.frames(root,times)]

def counts(a,b):
    return [int(np.any(x[24:] != y[24:],axis=2).sum()) for x,y in zip(a,b)]

control = ["size 608 432","focal 670","camera 3222 3219 700 1800 1900 340","pin",
           "npc 7542 3221 3219 0 seq=7482@10","player 3222 3218 0 1024 "+fx.PLAYER_ITEMS,
           "proj 660 3218,3214 3222,3218 0 0 10 70 21 0","spot 659 3222 3218 0 75"]
times = [0,40,125]
a = capture(stock,control,times)
b = capture(local,control,times)
result = {"plane0Cycles":times, "plane0DifferentPixels":counts(a,b)}
assert a and all(np.array_equal(x,y) for x,y in zip(a,b))
lines = play.scene("repair_sparks")[0]
times = [30+c for c in [0,25,50,75,95,115,125,140,165,190,220,255,290]]
full = capture(local,lines,times)
result["repairCycles"] = times
for prefix in ("npc ","player ","proj ","spot "):
    result[prefix.strip()+"VisiblePixels"] = counts(full,capture(local,[line for line in lines if not line.startswith(prefix)],times))
    assert max(result[prefix.strip()+"VisiblePixels"]) > 0, prefix
(HERE/"visibility.json").write_text(json.dumps(result,indent=2))
print(result)
