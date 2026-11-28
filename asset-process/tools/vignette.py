"""Several pieces standing together on terrain, as they would in a room: the test of whether a set agrees with itself.

usage: vignette.py OUT "entry;entry;..." "X Z YAW DIST ELEV UP" ["cam" ...] [--ambient A --contrast C]
entry = WHAT:x:z:rot[:sx:sz]   WHAT is a real object id, a negative real NPC id (static figure), or the path of one of
our .bin models; x,z are tiles, rot is quarter turns (0..3), sx,sz the footprint in tiles for ours (default 1x1).
Each camera writes OUT/camN.png (the full 1280x900 capture).
"""
import sys, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import scene, author

args = sys.argv[1:]
amb, con = 30, 0
if '--ambient' in args:
    i = args.index('--ambient'); amb = int(args[i + 1]); del args[i:i + 2]
if '--contrast' in args:
    i = args.index('--contrast'); con = int(args[i + 1]); del args[i:i + 2]
out = pathlib.Path(args[0]); entries = [e for e in args[1].split(';') if e]; cams = args[2:]
real, extra = [], []
mid = 1246000
for e in entries:
    p = e.rsplit(':', 5) if e.count(':') >= 6 else None
    parts = e.split(':')
    # a Windows path contains one colon after the drive letter; rebuild it
    if len(parts[0]) == 1 and parts[0].isalpha():
        what = parts[0] + ':' + parts[1]; rest = parts[2:]
    else:
        what = parts[0]; rest = parts[1:]
    x, z, rot = int(rest[0]), int(rest[1]), int(rest[2])
    sx, sz = (int(rest[3]), int(rest[4])) if len(rest) >= 5 else (1, 1)
    if what.lstrip('-').isdigit():
        real.append((int(what), x, z, rot))
    else:
        extra.append((mid, pathlib.Path(what).read_bytes(), author.object_def(mid, sx=sx, sz=sz, ambient=amb, contrast=con), x, z, rot)); mid += 1
content = scene.build(out, None, real, extra)
for k, cam in enumerate(cams):
    print(scene.capture(content, f'cam{k}', cam.split()))
