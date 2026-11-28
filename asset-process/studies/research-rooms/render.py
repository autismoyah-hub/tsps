"""Draw real rooms from the map, re-placed tile for tile (every object with its own definition, shape and rotation) on
a flat floor in the same renderer and with the same cameras as our rooms; and draw plans.

usage: render.py OUTDIR ROOMID [ROOMID ...]        writes OUTDIR/ROOMID/{overview,played,threeq}.png and plan.png
The floor is flat: real rooms stand on the map's terrain (heights, overlay floor textures), which this does not carry;
room tiles are drawn in a lighter floor material, the tiles round them darker, so the found bounds can be checked.
Roofs are left out; walls, doors, wall decorations (both faces), floor decorations and everything standing are drawn.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, json, os, hashlib, zlib, pathlib, shutil, subprocess
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import rlib as R
os.environ.setdefault('SCENE_SNAPSHOT_BIN', str(_AP.bindir() / 'authored_scene_snapshot.exe'))
import scene
from PIL import Image, ImageDraw

DUMMY = R.TF / 'builds' / 'room-observatory' / 'kit' / 'boards_0.bin'
CENTRE = 44


def build(out, placements, floor_tiles, cx, cy):
    """placements: (definition bytes, x, z, rot, shape) in scene tiles; floor_tiles: set of scene tiles marked as room."""
    out = pathlib.Path(out)
    if out.exists(): shutil.rmtree(out)
    pkg = out / 'package'; pkg.mkdir(parents=True)
    rows = [('model', 1246000, 'x1246000.bin', DUMMY.read_bytes())]
    lines = ['BROKEN_WATER_RETURN 1']
    room_v = set()
    for x, z in floor_tiles:
        room_v |= {(x, z), (x + 1, z), (x, z + 1), (x + 1, z + 1)}
    for x in range(128):
        for z in range(128):
            lines.append(f'V {x} {z} 0 {5806 if (x, z) in room_v else 5803} 1')
    nid = 65000
    for d, x, z, rot, shape in placements:
        rows.append(('object', nid, f'x{nid}.bin', d))
        lines.append(f'O {nid} {x} {z} {shape} {rot}')
        nid += 1
    reg = 'TOWER_ASSET_PACKAGE 1\n' + ''.join(f'{k} {i} {n} {zlib.crc32(b):08x}\n' for k, i, n, b in rows)
    regb = reg.encode(); (pkg / 'registry.txt').write_bytes(regb)
    for k, i, n, b in rows: (pkg / n).write_bytes(b)
    floor = ('\n'.join(lines) + '\n').encode(); (out / 'floor.bwr').write_bytes(floor)
    bind = b'CINDERWORKS_VISUAL_BINDINGS 1\n'; (out / 'bindings.txt').write_bytes(bind)
    sha = lambda b: hashlib.sha256(b).hexdigest()
    (out / 'content.txt').write_text('GIELINOR_AUTHORED_CONTENT 1\n'
        f'package=package\npackage_sha256={scene.package_identity(regb, [b for *_, b in rows])}\n'
        f'floor=floor.bwr\nfloor_sha256={sha(floor)}\nbindings=bindings.txt\nbindings_sha256={sha(bind)}\nmotion=off\n', newline='\n')
    return out / 'content.txt'


def cams(w, h):
    s = max(w, h)
    d = int(1440 * max(s, 7) / 9)
    return {'overview': (CENTRE + .5, CENTRE + .5, 0, d, int(d * 1280 / 1440), 65),
            'threeq': (CENTRE + .5, CENTRE + .5, 14336, int(d * 1510 / 1440), int(d * 1190 / 1440), 65),
            'played': (CENTRE + .5, CENTRE, 0, int(1024 * max(s, 7) / 9), int(896 * max(s, 7) / 9), 60)}


def room_scene(r, out, include_outer=True):
    D = R.defs()
    xs = [t[0] for t in r['tiles']]; ys = [t[1] for t in r['tiles']]
    ox = CENTRE - (min(xs) + max(xs)) // 2; oy = CENTRE - (min(ys) + max(ys)) // 2
    pl = []
    for oid, x, y, sh, rot, k in r['items']:
        if k.endswith(':outer') and not include_outer: continue
        d = R.resolved(oid)
        if not d or not d.get('models'): continue
        pl.append((d['raw'], x + ox, y + oy, rot, sh))
    tiles = {(x + ox, y + oy) for x, y in r['tiles']}
    return build(out, pl, tiles, CENTRE, CENTRE), (r['w'], r['h'])


COL = {'furniture': (176, 120, 60), 'light': (255, 220, 80), 'clutter': (120, 200, 120), 'stairs': (150, 150, 220),
       'scenery-unnamed': (120, 100, 90)}


def plan(norm, path, title='', cell=22):
    tiles = [tuple(t) for t in norm['tiles']]
    xs = [t[0] for t in tiles]; ys = [t[1] for t in tiles]
    x0, y0, x1, y1 = min(xs) - 1, min(ys) - 1, max(xs) + 1, max(ys) + 1
    W, H = (x1 - x0 + 1) * cell, (y1 - y0 + 1) * cell
    im = Image.new('RGB', (max(W, 200), H + 18), (30, 30, 34)); dr = ImageDraw.Draw(im)
    P = lambda x, y: ((x - x0) * cell, (y1 - y) * cell)
    S = set(tiles)
    for x, y in tiles:
        a = P(x, y); dr.rectangle([a[0], a[1], a[0] + cell - 1, a[1] + cell - 1], fill=(88, 84, 80))
    floor = [o for o in norm['objects'] if o['cls'] == 'floor']
    for o in floor:
        a = P(o['x'], o['y']); dr.rectangle([a[0] + 3, a[1] + 3, a[0] + cell - 4, a[1] + cell - 4], outline=(150, 60, 160) if o['name'] != 'null' else (110, 100, 110))
    doors = {tuple(d) for d in norm['doors']}
    for x, y in tiles:
        for d, (dx, dy) in enumerate([(-1, 0), (0, 1), (1, 0), (0, -1)]):
            if (x + dx, y + dy) in S: continue
            a = P(x, y)
            seg = {0: [a[0], a[1], a[0], a[1] + cell], 2: [a[0] + cell, a[1], a[0] + cell, a[1] + cell],
                   1: [a[0], a[1], a[0] + cell, a[1]], 3: [a[0], a[1] + cell, a[0] + cell, a[1] + cell]}[d]
            dr.line(seg, fill=(80, 220, 255) if (x, y, d) in doors else (235, 235, 235), width=3)
    for o in norm['objects']:
        if o['cls'] in COL:
            a = P(o['x'], o['y'] + o['d'] - 1)
            dr.rectangle([a[0] + 2, a[1] + 2, a[0] + o['w'] * cell - 3, a[1] + o['d'] * cell - 3], fill=COL[o['cls']], outline=(0, 0, 0))
        elif o['cls'] in ('decor', 'light-wall', 'window') and o.get('inside', True):
            a = P(o['x'], o['y']); c = (230, 60, 60) if o['cls'] == 'decor' else (255, 220, 80) if o['cls'] == 'light-wall' else (80, 160, 255)
            cxp, cyp = a[0] + cell // 2, a[1] + cell // 2
            off = {0: (-cell // 2 + 4, 0), 1: (0, -cell // 2 + 4), 2: (cell // 2 - 4, 0), 3: (0, cell // 2 - 4)}[o['rot'] % 4]
            dr.ellipse([cxp + off[0] - 4, cyp + off[1] - 4, cxp + off[0] + 4, cyp + off[1] + 4], fill=c)
    dr.text((3, H + 3), title[:40], fill='white')
    im.save(path)
    return im


def main():
    out = pathlib.Path(sys.argv[1]); out.mkdir(parents=True, exist_ok=True)
    rooms = json.loads((R.HERE / 'rooms.json').read_text())
    for rid in map(int, sys.argv[2:]):
        r = rooms[rid]
        d = out / str(rid); d.mkdir(exist_ok=True)
        content, (w, h) = room_scene(r, d / 'scene')
        for name, cam in cams(w, h).items():
            p = scene.capture(content, name, [str(c) for c in cam])
            shutil.copyfile(p, d / (name + '.png'))
        print(rid, 'drawn', d)


if __name__ == '__main__':
    main()
