"""compose_arena.py: a FIRST arena for the Warden-General, composed to ARENAS.md from pieces that already exist.

usage: compose_arena.py TAG [--floor boards|flags] [--no-board]
writes render/research-arenas-warden-TAG/: top.png, top-empty.png (straight down, with and without the boss),
played.png, threeq.png, placements.csv (the project's compose table), floor.json (the floor description arenacheck.py
reads), check.txt (arenacheck's report) and board.jpg (ours beside Tekton's room and the Nightmare's arena, the same
three cameras). Copies placements.csv, floor.json and check.txt to this folder as warden-arena-TAG.*.

The place: the top of the astronomers' tower. A 17 x 17 court (floor 277 tiles before the edge pieces) with splayed
corners, the observatory's own boards with their border course, its 5 x 5 star dial in the middle under the boss, the
observatory's masonry walls with windows, a door in the south wall (the entry), and at the edge what exists: suits of
armour as the statues, braziers in the corners, telescopes, two standards on the far wall on the door's line and a
shield on each side wall. Nothing stands on the fighting floor.

HONEST NOTES. The boss is his own model (builds/boss-warden-general-3) placed as a 4 x 4 static object in its rest pose,
lit as scenery by the room renderer (tools/scene.py's own way of standing a figure in a room: scene.npc_as_object);
in the real pictures the boss is an NPC drawn by the client's actor path. The south wall is cut away for the played and
three-quarter cameras, as the project's rooms are drawn (the game would hide it or the camera would stand inside).
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
from pathlib import Path
import argparse, csv, json, os, shutil, subprocess, sys
sys.dont_write_bytecode = True
HERE = Path(__file__).resolve().parent
TF = HERE.parents[1]
B = TF / 'builds'
OBS = B / 'room-observatory'
os.environ.setdefault('SCENE_SNAPSHOT_BIN', str(_AP.bindir() / 'authored_scene_snapshot.exe'))
sys.path.insert(0, str(OBS)); sys.path.insert(0, str(TF / 'tools')); sys.path.insert(0, str(HERE))
from roomlib import Room
import osrscache as O, shots
from PIL import Image, ImageDraw

UNDER = 0                               # no ground outside the court: the tower top stands in the dark, as instanced arenas do (5803 = the rooms' grey backdrop)
N = 17; O0 = 36; C = N // 2            # tiles a side, world tile of the court's south-west corner tile, index of the middle tile
KIT = OBS / 'kit'; ARM = B / 'room-armoury' / 'kit'; DARK = B / 'room-observatory-furnished' / 'kit-dark'
BOSS = B / 'boss-warden-general-3' / 'boss-warden-general-3.bin'
BOSS_TILE = (O0 + C - 1, O0 + C - 1)   # south-west tile of his 4 x 4: tiles 43..46, as near the middle as an even boss can stand in an odd court
ENTRY = (O0 + C, O0)                   # the tile inside the south door
PLAYER = (O0 + C, O0 + 2)
P = {'stand-a': 'original-arm-stand-a/arm-stand-a.bin', 'stand-b': 'original-arm-stand-b/arm-stand-b.bin',
     'brazier': 'original-arm2-brazier-a/arm2-brazier-a.bin', 'telescope': 'original-obs1-telescope-a/obs1-telescope-a.bin',
     'standard': 'original-arm2-standard-b/arm2-standard-b.bin', 'shield': 'original-arm2-shield-a/arm2-shield-a.bin',
     'torch': 'original-arm2-torch-a/arm2-torch-a.bin'}
NAMES = {'stand': ('Suit of armour', 'furniture', 'rack'), 'brazier': ('Brazier', 'light', 'light'), 'telescope': ('Telescope', 'furniture', 'machine'),
         'standard': ('Standard', 'decor', 'decor'), 'shield': ("Wardens' shield", 'decor', 'decor'), 'torch': ('Wall torch', 'light-wall', 'light')}
BACK = {0: (0, 1), 1: (1, 0), 2: (0, -1), 3: (-1, 0)}     # world direction of a piece's back for each rotation (front faces south at 0)

# standing and hung pieces: (piece, x, z, rotation, shape) in court tiles 0..16
EDGE = []
for a in (4, 12):                      # two suits of armour a side, mirrored about the middle line: the "statues" of the edge
    EDGE += [('stand-a', a, N - 1, 0, 10), ('stand-b', a, 0, 2, 10), ('stand-a', 0, a, 3, 10), ('stand-b', N - 1, a, 1, 10)]
EDGE += [('brazier', 1, 1, 0, 10), ('brazier', N - 2, 1, 0, 10), ('brazier', 1, N - 2, 0, 10), ('brazier', N - 2, N - 2, 0, 10)]   # light in the four corners
EDGE += [('telescope', 6, N - 1, 0, 10), ('telescope', 10, N - 1, 0, 10)]                                                      # the tower's own instruments, far wall
EDGE += [('standard', 7, N - 1, 1, 4), ('standard', 9, N - 1, 1, 4)]                                                           # the far wall on the door's line
EDGE += [('shield', 0, C, 0, 4), ('shield', N - 1, C, 2, 4)]                                                                   # one on each side wall, on the cross line


def turn(x, z, q):
    for _ in range(q): x, z = z, N - 1 - x
    return x, z


def extents(path):
    v = O.decode_model(Path(path).read_bytes())['vertices']
    return max(p[2] for p in v)


def court(cut='', floor='boards', boss=True):
    room = Room(30, 0); rows = []
    FK = DARK if floor.startswith('dark') else KIT            # the boards and the dial
    WK = DARK if floor == 'dark' else KIT                     # the walls

    def place(kit, name, x, z, q, shape, **kw):
        path = kit / (name + '.bin')
        lights = {'ambient': 40, 'contrast': 25} if shape == 0 else {}
        room.kit(path, O0 + x, O0 + z, q, shape, **lights, **kw)
        rows.append([str(path), O0 + x, O0 + z, q, shape, 1, 1, 0, 0, '', '', ''])
    corner = {}
    for q, sides in enumerate(('sw', 'wn', 'ne', 'es')):
        for x, z, key in [(1, 0, 'toe'), (0, 1, 'heel'), (1, 1, 'joint')]:
            a, b = turn(x, z, q); corner[a, b] = (q, key)
            if not (cut and 's' in sides): place(WK, 'splay_' + key, a, b, q, 0)
            else: rows.append([str(KIT / ('splay_' + key + '.bin')), O0 + a, O0 + b, q, 0, 1, 1, 0, 0, '', '', ''])
    for x in range(N):
        for z in range(N):
            if x in (0, N - 1) and z in (0, N - 1): continue
            q, key = corner.get((x, z), (0, ''))
            dx, dz = x - C, z - C
            if floor != 'flags':
                if key in ('toe', 'heel'): place(FK, 'boards_mitre', x, z, q, 22)
                elif key == 'joint': place(FK, 'boards_return', x, z, q, 22)
                elif x in (0, N - 1) or z in (0, N - 1):
                    rot = 0 if x == 0 else 2 if x == N - 1 else 1 if z == N - 1 else 3
                    place(FK, 'boards_crossedge' if rot % 2 else 'boards_edge', x, z, rot, 22)
                elif (FK / f'dial_{dx}_{dz}.bin').exists(): place(FK, f'dial_{dx}_{dz}', x, z, 0, 22)
                else: place(FK, 'boards_' + str((x * 7 + z * 5) % 3), x, z, 0, 22)
            else:
                if key in ('toe', 'heel'): place(ARM, 'arm-flag-cut', x, z, q, 22)
                elif max(abs(dx), abs(dz)) == 1: place(ARM, f'arm-flag-ring-{dx}-{dz}', x, z, 0, 22)
                else: place(ARM, 'arm-flag-' + str((x + 2 * z) % 3), x, z, (x // 2 + z // 3) % 4, 22)
    for q, side in enumerate('wnes'):
        for i in range(2, N - 2):
            x, z = turn(0, i, q)
            loc = x if side in 'ns' else z
            name = 'door' if (side == 's' and loc == C) else 'window' if loc in (2, C, N - 3) else ('wall_b' if i % 2 else 'wall_a')
            if side in cut: rows.append([str(KIT / (name + '.bin')), O0 + x, O0 + z, q, 0, 1, 1, 0, 0, '', '', '']); continue
            place(WK, name, x, z, q, 0)
    for name, x, z, rot, shape in EDGE:
        path = B / P[name]; kind = name.split('-')[0]; nm, cls, func = NAMES[kind]
        if shape == 4:
            if 'wnes'[rot] not in cut: room.kit(path, O0 + x, O0 + z, rot, 4, ambient=40, contrast=25)
            rows.append([str(path), O0 + x, O0 + z, rot, 4, 1, 1, 0, 0, nm, cls, func]); continue
        room.kit(path, O0 + x, O0 + z, rot, 10, sx=1, sz=1, ambient=30, contrast=0)
        dx = dz = 0; back = BACK[rot]
        at_wall = (back == (0, 1) and z == N - 1) or (back == (0, -1) and z == 0) or (back == (-1, 0) and x == 0) or (back == (1, 0) and x == N - 1)
        if at_wall:
            pull = max(0, int(extents(path)) - 30); dx, dz = -back[0] * pull, -back[1] * pull
        if kind == 'brazier':            # a corner against the splay: step off it
            dx += 14 if x == 1 else -14; dz += 14 if z == 1 else -14
        mid, mesh, definition, *placement = room.extra[-1]
        off = bytes([70]) + dx.to_bytes(2, 'big', signed=True) + bytes([71]) + (-3).to_bytes(2, 'big', signed=True) + bytes([72]) + dz.to_bytes(2, 'big', signed=True)
        room.extra[-1] = (mid, mesh, off + definition, *placement)
        rows.append([str(path), O0 + x, O0 + z, rot, 10, 1, 1, dx, dz, nm, cls, func])
    if boss:
        room.kit(BOSS, BOSS_TILE[0], BOSS_TILE[1], 0, 10, sx=4, sz=4, ambient=14, contrast=5)
        mid, mesh, definition, *placement = room.extra[-1]
        room.extra[-1] = (mid, mesh, bytes([71]) + (-3).to_bytes(2, 'big', signed=True) + definition, *placement)
    rows.append(['boss:' + str(BOSS), BOSS_TILE[0], BOSS_TILE[1], 0, 10, 4, 4, 0, 0, 'Warden-General', 'boss', 'boss'])
    return room, rows


def cams():
    cx = cy = O0 + N / 2
    Cm = shots.cams(cx, cy, N + 4, PLAYER)
    bx, by = BOSS_TILE[0] + 2, BOSS_TILE[1] + 2
    tx, ty = PLAYER[0] + .5 + (bx - PLAYER[0] - .5) * 0.45, PLAYER[1] + .5 + (by - PLAYER[1] - .5) * 0.45
    Cm['played'] = (round(tx, 2), round(ty, 2), 0) + shots.PLAYED
    return Cm


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('tag'); ap.add_argument('--floor', default='boards', choices=['boards', 'dark', 'darkfloor', 'flags']); ap.add_argument('--no-board', action='store_true')
    a = ap.parse_args()
    out = TF / 'render' / f'research-arenas-warden-{a.tag}'
    if out.exists(): shutil.rmtree(out)
    out.mkdir(parents=True)
    Cm = cams()
    views = [('top', '', True), ('top-empty', '', False), ('played', 's', True), ('threeq', 'sw', True)]
    rows_full = None
    for name, cut, boss in views:
        room, rows = court(cut, a.floor, boss)
        room.build(out / f'scene-{name}', underlay=UNDER)
        shutil.copyfile(room.shoot(name, Cm[name.replace('-empty', '')]), out / f'{name}.png')
        if name == 'top': rows_full = rows
    with (out / 'placements.csv').open('w', newline='') as f:
        w = csv.writer(f); w.writerow(['source', 'tile_x', 'tile_z', 'rotation', 'shape', 'width', 'depth', 'shift_x', 'shift_z', 'name', 'cls', 'func'])
        w.writerows(rows_full)
    floor = dict(name=f"Warden-General's arena, first composition ({a.floor})", boss=[BOSS_TILE[0], BOSS_TILE[1], 4, 4], entries=[list(ENTRY)],
                 kinds=[['dial_', 'star dial'], ['boards_(edge|crossedge|return)', 'border boards'], ['boards_', 'boards'],
                        ['arm-flag-ring', 'brass ring flags'], ['arm-flag', 'flags']],
                 not_floor='flag-cut|mitre', wall_tile='splay',
                 pictures=dict(top='top.png', empty='top-empty.png', cam=list(Cm['top'])))
    (out / 'floor.json').write_text(json.dumps(floor, indent=1), encoding='utf-8')
    r = subprocess.run([sys.executable, '-I', str(HERE / 'arenacheck.py'), str(out / 'placements.csv'), str(out / 'floor.json'), '--json', str(out / 'check.json')],
                       capture_output=True, text=True)
    (out / 'check.txt').write_text(r.stdout + r.stderr, encoding='utf-8')
    print(r.stdout[-2600:], r.stderr[-800:])
    for n in ('placements.csv', 'floor.json', 'check.txt'):
        shutil.copyfile(out / n, HERE / f'warden-arena-{a.tag}.{n.split(".")[0] if n != "check.txt" else "check"}.{n.split(".")[-1]}')
    if not a.no_board:
        board(out, a.tag)
    print(out)


def board(out, tag, reals=('tekton', 'nightmare')):
    R = TF / 'render' / 'research-arenas'
    names = {'tekton': "REAL: Tekton's room, Chambers of Xeric (boss 4 tiles)", 'nightmare': "REAL: the Nightmare's arena (boss 5 tiles)"}
    cols = [('played', 'played camera'), ('threeq', 'three-quarter'), ('top', 'from above')]
    tw, th = 640, 450
    LH = 38
    im = Image.new('RGB', (tw * 3, (th + LH) * (1 + len(reals))), (24, 24, 28)); dr = ImageDraw.Draw(im)
    srcs = [("OURS: the Warden-General's arena, first composition from existing pieces (boss placed as a static object; south wall cut away)", out)] + [(names[k], k) for k in reals]
    for r, (label, src) in enumerate(srcs):
        for c, (v, vl) in enumerate(cols):
            p = (src / f'{v}.png') if isinstance(src, Path) else (R / f'{src}-{v}.png')
            t = Image.open(p).convert('RGB').resize((tw, th), Image.LANCZOS)
            im.paste(t, (c * tw, r * (th + LH) + LH))
            if c == 0: dr.text((6, r * (th + LH) + 4), label, fill=(255, 230, 120) if r == 0 else (230, 230, 230))
            dr.text((c * tw + 6, r * (th + LH) + 21), vl, fill=(170, 170, 170))
    im.save(out / 'board.jpg', quality=93)
    (HERE / 'boards').mkdir(exist_ok=True)
    im.save(HERE / 'boards' / f'board-warden-arena-{tag}.jpg', quality=93)


if __name__ == '__main__':
    main()
