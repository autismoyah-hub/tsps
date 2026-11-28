import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).parent))
from patchlib import Patch
P = Patch(_AP.fix(r'<ROOT>\tools\lineup.py'))
P.rep('''entry = real object id | -npc id | path to an authored model .bin
''', '''entry = real object id | -npc id | path to an authored model .bin      (placed as an ordinary object, shape 10)
      | ID@SHAPE[:ROT]     a real object placed with that shape, e.g. 2341@0 (a wall), 20624@22 (a floor piece)
      | PATH@SHAPE[:ROT]   one of ours placed with that shape; its definition lists the model for that shape only
                           (opcode 6: [6, 1] + model id + [SHAPE]) and is not solid
Shapes: 0 straight wall on one edge of its tile, 1 / 3 corner posts, 2 corner wall, 9 diagonal wall, 4 wall
decoration, 22 ground decoration (floor pieces), 10 ordinary object. A real object must list a model for the shape
asked for (tools/wire.py ID x.png prints the shapes it has; scene.object_json checks it here).
ROT is the placement rotation 0-3. For a wall, 0 = west edge of the tile, 1 = north, 2 = east, 3 = south. The
default is 1 for shapes 0 to 9, so that a wall stands on the far (north) edge and shows its room side to the
front view, and 0 for the others. The turned views (--views, --yaws) turn the camera, not the piece.
Known limit: a floor piece at height 0 is hidden by the terrain it lies in. Real floor objects sit at height 0,
so a real ID@22 is almost invisible on our terrain (only parts standing above the ground show); ours must be
built a unit or two above 0 to be seen here. --ambient / --contrast apply to ours of any shape.
''')
P.rep('''    x = X0 + k * SP
    if e.lstrip('-').isdigit():
        real.append((int(e), x, Z0, 0)); key.append(f'{k + 1}: real {e}')
    else:
        sx, sz = map(int, a.size.split('x'))
''', '''    x = X0 + k * SP
    what, _, placing = e.rpartition('@')
    if what and placing.replace(':', '').isdigit():
        # ID@SHAPE[:ROT] or PATH@SHAPE[:ROT]
        shape, _, rot = placing.partition(':')
        shape = int(shape); rot = int(rot) if rot else (1 if shape <= 9 else 0)
        if what.isdigit():
            j = scene.object_json(int(what), shape)       # stops with the list of shapes when it has no model for this one
            shaped.append((scene.object_def(int(what)), x, Z0, rot, shape))
            key.append(f'{k + 1}: real {what} {j.get("name")!r} as shape {shape} rotation {rot} (archive folder shape-{j["shape_folder"]})')
        else:
            d = (bytes([39, a.contrast & 255, 29, a.ambient & 255]) + bytes([6, 1]) + mid.to_bytes(4, 'big') + bytes([shape])
                 + bytes([2]) + b'Authored' + bytes([0, 14, 1, 15, 1, 17, 19, 0, 0]))
            extra.append((mid, pathlib.Path(what).read_bytes(), d, x, Z0, rot, shape))
            key.append(f'{k + 1}: OURS {what} as shape {shape} rotation {rot}'); mid += 1
    elif e.lstrip('-').isdigit():
        real.append((int(e), x, Z0, 0)); key.append(f'{k + 1}: real {e}')
    else:
        sx, sz = map(int, a.size.split('x'))
''')
P.rep('''real, extra, key = [], [], []
''', '''real, extra, key, shaped = [], [], [], []      # shaped: real objects placed with a shape other than 10
''')
P.rep('''content = scene.build(out, None, real, extra)
''', '''# scene.build wants a model with every extra entry; a real definition names its own cache models, so a model
# already in the scene is named again (it is written once and the real definition does not use it)
for d, x, z, rot, shape in shaped:
    extra.append((extra[0][0], extra[0][1], d, x, z, rot, shape))
content = scene.build(out, None, real, extra)
''')
P.save()
