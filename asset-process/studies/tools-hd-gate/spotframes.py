"""spotframes.py OURS.bin: per-frame posed overlap of a spot-2768 rebuild against the HD original (Teacher's diagnostic,
2026-11-28). Prints, for every frame and both views, the overlap and the visible pixel counts of real and ours."""
import pathlib
import sys

sys.path.insert(0, str(pathlib.Path(__file__).parent))
import numpy as np
import hdgate as H

ref = H.Reference(614, 'spot', 2768)
u = H.osrs.decode_model(pathlib.Path(sys.argv[1]).read_bytes())
_, poses = H.animation(ref.rev, ref.d['sequence'], u)
for f, (real, m) in enumerate(zip(ref.motion, poses)):
    ours = H.render(m, 242, ref.anim_cam)['lit'][[0, 5]]
    for view, (a, b) in enumerate(zip(real, ours)):
        ma, mb = (a != H.BG).any(2), (b != H.BG).any(2)
        print(f'frame {f:2d} view {view}: overlap {H.overlap(ma, mb):.3f}  real px {int(ma.sum()):6d}  ours px {int(mb.sum()):6d}')
