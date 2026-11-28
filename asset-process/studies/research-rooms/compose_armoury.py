"""The wardens' armoury recomposed to ROOMS.md, with the same shell and the same three cameras as the orchestrator's
compose (night/armoury/compose.py, copied and changed here; nothing in night/ or builds/room-armoury is written).

usage: compose_armoury.py TAG [--plan NAME]
writes render/research-rooms-armoury-TAG/: overview.png, three-quarter.png, played.png, review.jpg (the three side by
side), placements.csv (every placement, the armoury's table form), and compare.jpg (the orchestrator's room above,
this one below, same cameras).

Pieces: only pieces that already exist under builds/ (listed in PIECES). A piece standing on a wall tile is pulled
off the wall so that its back ends 30 units from the tile centre (the armoury masonry starts at 32), and every
standing piece is lifted 3 units onto the flags, as the orchestrator's compose did.
"""
from pathlib import Path
import argparse, csv, importlib.util, shutil, sys
from PIL import Image, ImageDraw

HERE = Path(__file__).resolve().parent
TF = HERE.parents[1]
BUILDS = TF / 'builds'
SHELL = BUILDS / 'room-armoury'
sys.dont_write_bytecode = True
sys.path.insert(0, str(SHELL))
spec = importlib.util.spec_from_file_location('armoury_shell', SHELL / 'compose.py')
shell = importlib.util.module_from_spec(spec); spec.loader.exec_module(shell)
sys.path.insert(0, str(TF / 'tools'))
import osrscache as O

P = {  # short name -> file under builds/
    'rack-a': 'original-arm-rack-a/arm-rack-a.bin', 'rack-b': 'original-arm-rack-b/arm-rack-b.bin',
    'stand-a': 'original-arm-stand-a/arm-stand-a.bin', 'stand-b': 'original-arm-stand-b/arm-stand-b.bin',
    'dummy-a': 'original-arm-dummy-a/arm-dummy-a.bin', 'dummy-b': 'original-arm-dummy-b/arm-dummy-b.bin',
    'bench-a': 'original-arm-bench-a/arm-bench-a.bin', 'bench-b': 'original-arm-bench-b/arm-bench-b.bin',
    'chest-a': 'original-arm-chest-a/arm-chest-a.bin', 'chest-b': 'original-arm-chest-b/arm-chest-b.bin',
    'pail-a': 'original-arm-pail-a/arm-pail-a.bin', 'pail-b': 'original-arm-pail-b/arm-pail-b.bin',
    'candle-a': 'original-obs3-candle-a/obs3-candle-a.bin', 'candle-b': 'original-obs3-candle-b/obs3-candle-b.bin',
    'stool-a': 'original-obs3-stool-a/obs3-stool-a.bin', 'barrel': 'barrel/barrel.bin', 'crate': 'crate-15704/crate.bin',
    'sconce': 'room-scribes-hall/kit/sconce.bin',
}
SIZE = {'rack': (2, 1), 'bench': (2, 1), 'chest': (2, 1)}

# Plans: (piece, x, z, rotation, shape). Rotation: front faces south 0, west 1, north 2, east 3. Shape 4 = wall decoration
# on that tile's wall edge (rotation = the edge: 0 west, 1 north, 2 east, 3 south). Tiles x 40..48, z 40..48; door east at (48, 44).
PLANS = {
    'r1': [
        # north wall, the show wall seen by every camera: two weapon racks flanking the central slit (x 44)
        ('rack-a', 42, 48, 0, 10), ('rack-b', 45, 48, 0, 10),
        # corners against the splayed walls: a suit in each north corner
        ('stand-b', 41, 47, 0, 10), ('stand-b', 47, 47, 0, 10),
        # west wall, opposite the door: three suits on the door's line, candle stands between them
        ('stand-a', 40, 42, 3, 10), ('candle-a', 40, 43, 3, 10), ('stand-a', 40, 44, 3, 10), ('candle-a', 40, 45, 3, 10),
        ('stand-a', 40, 46, 3, 10),
        # east wall: two arms chests flanking the door, along the wall
        ('chest-a', 48, 42, 1, 10), ('chest-b', 48, 45, 1, 10),
        # south side: the work corner (grindstone, its pail, a stool, a barrel in the corner) and the training pair
        ('bench-b', 42, 40, 2, 10), ('pail-a', 44, 40, 2, 10), ('stool-a', 42, 41, 2, 10), ('barrel', 41, 41, 0, 10),
        ('dummy-a', 46, 40, 2, 10), ('dummy-b', 47, 41, 1, 10),
        # wall decorations: candle brackets flanking the door and over the outer suits and the pail
        ('sconce', 48, 42, 2, 4), ('sconce', 48, 46, 2, 4), ('sconce', 40, 42, 0, 4), ('sconce', 40, 46, 0, 4),
        ('sconce', 44, 40, 3, 4),
    ],
}
# r2: r1 without the two candle stands (real armouries line their suits at intervals, alone, and carry few lights:
# r1 sat above the real armouries' p90 on lights, wall lining and group size), and without the bracket on the
# cut-away south wall. Four brackets remain: the only wall decoration piece that exists (see ROOMS.md, missing pieces).
PLANS['r2'] = [p for p in PLANS['r1'] if p[0] != 'candle-a' and not (p[0] == 'sconce' and p[1:3] == (44, 40))]
VIEWS = {
    'overview': ('s', (44.5, 44.5, 0, 1440, 1280, 65)),
    'three-quarter': ('sw', (44.5, 44.5, 14336, 1510, 1190, 65)),
    'played': ('s', (44.5, 44.0, 0, 1024, 896, 60)),
}
BACK = {0: (0, 1), 1: (1, 0), 2: (0, -1), 3: (-1, 0)}     # world direction of a piece's back for each rotation


def extents(path):
    v = O.decode_model((BUILDS / path).read_bytes())['vertices']
    return max(p[2] for p in v)          # how far the back reaches behind the tile centre at rotation 0


def wall_dirs(x, z):
    """World directions in which a wall stands on this tile's edge (the armoury shell: x 40 west, 48 east, z 40 south, 48 north)."""
    d = []
    if x == 40: d.append((-1, 0))
    if x == 48: d.append((1, 0))
    if z == 40: d.append((0, -1))
    if z == 48: d.append((0, 1))
    return d


def furnished(cut, plan):
    room = shell.chamber(cut)
    rows = []
    for name, x, z, rot, shape in PLANS[plan]:
        path = BUILDS / P[name]
        kind = name.split('-')[0]
        sx, sz = SIZE.get(kind, (1, 1))
        if shape == 4:
            side = {0: 'w', 1: 'n', 2: 'e', 3: 's'}[rot]
            if side not in cut:          # a cut-away wall carries no decoration in the cutaway views
                room.kit(path, x, z, rot, 4, ambient=40, contrast=25)
            rows.append((P[name], x, z, rot, 4, 1, 1, 0, 0)); continue
        room.kit(path, x, z, rot, shape, sx=sx, sz=sz, ambient=30, contrast=0)
        dx = dz = 0
        back = BACK[rot]
        fw, fd = (sz, sx) if rot % 2 else (sx, sz)
        if any(back in wall_dirs(x + i, z + j) for i in range(fw) for j in range(fd)):
            pull = max(0, extents(P[name]) - 30)
            dx, dz = -back[0] * pull, -back[1] * pull
        if (x, z) in ((41, 47), (47, 47), (41, 41), (47, 41)):          # a corner against the splay: step off it
            dx += 10 if x == 41 else -10; dz += 10 if z == 41 else -10
        mid, mesh, definition, *placement = room.extra[-1]
        offsets = bytes([70]) + dx.to_bytes(2, 'big', signed=True) + bytes([71]) + (-3).to_bytes(2, 'big', signed=True)
        offsets += bytes([72]) + dz.to_bytes(2, 'big', signed=True)
        room.extra[-1] = (mid, mesh, offsets + definition, *placement)
        rows.append((P[name], x, z, rot, shape, sx, sz, dx, dz))
    return room, rows


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument('tag'); ap.add_argument('--plan', default='r1')
    a = ap.parse_args()
    if not a.tag.replace('-', '').isalnum(): ap.error('tag must be alphanumeric or hyphenated')
    out = TF / 'render' / f'research-rooms-armoury-{a.tag}'
    out.mkdir(parents=True, exist_ok=False)
    scenes = {}
    for cut in ('', 's', 'sw'):
        room, rows = furnished(cut, a.plan)
        room.build(out / ('scene-' + (cut or 'full')))
        scenes[cut] = (room, rows)
    # the placement table, the armoury's own form: shell rows from the full scene plus furniture with its offsets
    room, rows = scenes['']
    by_id = {mid: key[0] for key, (mid, *_) in room._models.items()}
    with (out / 'placements.csv').open('w', newline='') as f:
        w = csv.writer(f); w.writerow(['source', 'tile_x', 'tile_z', 'rotation', 'shape', 'width', 'depth', 'shift_x', 'shift_z'])
        furn = {(str(BUILDS / r[0]), r[1], r[2], r[3], r[4]): r for r in rows}
        for mid, _, _, x, z, rot, shape in room.extra:
            src = by_id[mid]; r = furn.get((str(Path(src)), x, z, rot, shape))
            w.writerow([src, x, z, rot, shape, r[5] if r else 1, r[6] if r else 1, r[7] if r else 0, r[8] if r else 0])
    shutil.copyfile(out / 'placements.csv', HERE / f'armoury-{a.tag}-placements.csv')
    board = Image.new('RGB', (1440, 365)); d = ImageDraw.Draw(board)
    for i, (name, (cut, cam)) in enumerate(VIEWS.items()):
        frame = scenes[cut][0].shoot(name, cam)
        shutil.copyfile(frame, out / (name + '.png'))
        im = Image.open(frame).convert('RGB'); im.thumbnail((480, 338)); board.paste(im, (i * 480, 0))
        d.text((i * 480 + 12, 342), name + ' (recomposed)', fill='white')
    board.save(out / 'review.jpg', quality=94)
    orig = TF / 'night/armoury/boards/final-selected/review.jpg'
    cmp = Image.new('RGB', (1440, 730)); cmp.paste(Image.open(orig).convert('RGB').resize((1440, 365)), (0, 0)); cmp.paste(board, (0, 365))
    ImageDraw.Draw(cmp).text((12, 6), 'ABOVE: as the orchestrator composed it (graded Usable)   BELOW: recomposed to ROOMS.md (not yet seen by the owner)', fill='yellow')
    cmp.save(out / 'compare.jpg', quality=94)
    print(out)


if __name__ == '__main__':
    main()
