"""A line-up in the game's classic look: every entry captured alone, from the same camera, then tiled.

usage: lineup.py OUT "entry,entry,..." [--shuffle SEED] [--views N | --yaws 0,45,315,180] [--dist D] [--elev E]
       [--crop PX] [--size 2x1] [--ambient A --contrast C] [--small] [--aim DX,DZ] [--up H]
--aim DX,DZ moves the point every camera looks at (and turns about) by that many game units east and north of
the middle of each entry's tile; --up H is that point's height. Default: the tile middle, 45 up, which suits a
piece standing on the floor. A wall decoration hangs on the tile's edge, well above 45: gate.py aims at the
middle of the real piece as mounted (wall decoration at rotation 1: north edge, so about --aim 0,58 --up 140).
A lower --elev with the same --dist gives a lower camera (for checking draw order under overhangs).
entry = real object id | -npc id | path to an authored model .bin      (placed as an ordinary object, shape 10)
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
Writes OUT/lineup.png (positions numbered) and OUT/key.txt (which is which; keep it from the judge).
Each entry stands on terrain, centred in its own capture, so perspective does not favour any position.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, pathlib, random, argparse
sys.path.insert(0, str(pathlib.Path(__file__).parent))
import scene, author
from PIL import Image, ImageDraw, ImageFont

ap = argparse.ArgumentParser()
ap.add_argument('out'); ap.add_argument('entries')
ap.add_argument('--shuffle', type=int); ap.add_argument('--views', type=int, default=1)
ap.add_argument('--dist', type=float, default=330); ap.add_argument('--elev', type=float, default=250)
ap.add_argument('--up', type=float, default=45); ap.add_argument('--crop', type=int, default=440)
ap.add_argument('--ambient', type=int, default=10); ap.add_argument('--contrast', type=int, default=30)
ap.add_argument('--size', default='1x1', help='tiles our object covers, e.g. 2x1, so it stands where a real one of that size stands')
ap.add_argument('--yardstick', type=int, default=0, help='real NPC id placed beside every entry for scale')
ap.add_argument('--yaws', help='comma list of views in degrees turned from the front, e.g. 0,45,315,180; overrides --views')
ap.add_argument('--small', action='store_true', help='also write lineup-small.png at about game size (one third)')
ap.add_argument('--aim', default='0,0', help='DX,DZ in game units: move the camera target off the tile middle (east, north)')
a = ap.parse_args()
out = pathlib.Path(a.out)
entries = a.entries.split(',')
if a.shuffle is not None:
    random.Random(a.shuffle).shuffle(entries)
SP, X0, Z0 = 8, 20, 60
real, extra, key, shaped = [], [], [], []      # shaped: real objects placed with a shape other than 10
mid = 1246000
for k, e in enumerate(entries):
    x = X0 + k * SP
    what, _, placing = e.rpartition('@')
    if what and placing.replace(':', '').isdigit():
        # ID@SHAPE[:ROT] or PATH@SHAPE[:ROT]
        shape, _, rot = placing.partition(':')
        shape = int(shape); rot = int(rot) if rot else (1 if shape <= 9 else 0)
        if what.isdigit():
            j = scene.object_json(int(what), shape, quiet=True)   # stops with the list of shapes when it has no model for this one
            if j['shapes_listed'] is None and shape not in (10, 11):
                raise SystemExit(f'real object {what} has untyped models, which the game draws only as shape 10: {e} would be an empty cell (tested)')
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
        extra.append((mid, pathlib.Path(e).read_bytes(), author.object_def(mid, sx=sx, sz=sz, ambient=a.ambient, contrast=a.contrast), x, Z0, 0)); key.append(f'{k + 1}: OURS {e}'); mid += 1
    if a.yardstick:
        real.append((-a.yardstick, x + 1, Z0, 0))
if not extra:
    extra.append((mid, pathlib.Path(_AP.fix(r'<ROOT>\examples\barrel\barrel.bin')).read_bytes(), author.object_def(mid), 2, 2, 0))
# scene.build wants a model with every extra entry; a real definition names its own cache models, so a model
# already in the scene is named again (it is written once and the real definition does not use it)
for d, x, z, rot, shape in shaped:
    extra.append((extra[0][0], extra[0][1], d, x, z, rot, shape))
content = scene.build(out, None, real, extra)
(out / 'key.txt').write_text('\n'.join(key) + '\n')
yaws = [0, 2048 * 3, 2048 * 5, 8192][:a.views]          # front, and turned views
if a.yaws:
    yaws = [round(float(d) % 360 / 360 * 16384) for d in a.yaws.split(',')]   # 45 and 315 are the front diagonals
c = min(a.crop, 900)                                    # a capture is 1280 x 900
AIM = [float(v) for v in a.aim.split(',')]              # 0,0 leaves every pose exactly as before
try: font = ImageFont.truetype('segoeuib.ttf', 34)
except Exception: font = ImageFont.load_default()
board = Image.new('RGB', (len(entries) * c, len(yaws) * c), (0, 0, 0))
d = ImageDraw.Draw(board)
for k in range(len(entries)):
    cx = X0 + k * SP + (1.0 if a.yardstick else int(a.size.split('x')[0]) / 2)   # centre of the footprint
    for r, yaw in enumerate(yaws):
        png = scene.capture(content, f'e{k}v{r}', [cx + AIM[0] / 128, Z0 + 0.5 + AIM[1] / 128, yaw, a.dist, a.elev, a.up])
        im = Image.open(png).convert('RGB')
        w, h = im.size
        board.paste(im.crop(((w - c) // 2, (h - c) // 2, (w + c) // 2, (h + c) // 2)), (k * c, r * c))
    d.text((k * c + 14, 8), str(k + 1), fill=(240, 240, 240), font=font)
board.save(out / 'lineup.png')
if a.small:
    board.resize((board.width // 3, board.height // 3), Image.LANCZOS).save(out / 'lineup-small.png')
print(out / 'lineup.png', board.size)
