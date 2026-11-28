"""draw.py: draw the real arenas of arenas.py with their bosses, through shots.py (the client's own loader and painter).

  draw.py [KEY ..]        writes render/research-arenas/KEY-{top,top-empty,played,threeq}.png and KEY-check.png
                          (the top view with the measured floor outlined in yellow and the boss's tiles in cyan: the
                          proof that the flood fill found the floor a player fights on)
Importable: floor_of(a) -> (World, floor set), project(cam, size) -> function world (x, y, height) -> pixel.
"""
import sys, math, pathlib
sys.dont_write_bytecode = True
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import arenalib as A, shots
from arenas import ARENAS, BY
from PIL import Image, ImageDraw
OUT = A.TF / 'render' / 'research-arenas'


def floor_of(a, margin=14):
    sx, sy = a['seed']; r = 40
    W0 = A.World(sx - r, sy - r, sx + r, sy + r, a['plane'])
    S = W0.floor(a['seed'], a.get('clip'), a.get('cuts', ()))
    return W0, S


def bbox(S):
    xs = [t[0] for t in S]; ys = [t[1] for t in S]
    return min(xs), min(ys), max(xs), max(ys)


def centre(S):
    x0, y0, x1, y1 = bbox(S)
    return (x0 + x1 + 1) / 2, (y0 + y1 + 1) / 2, max(x1 - x0 + 1, y1 - y0 + 1)


def ground(W, x, y):
    """Height at a tile corner (the tile's own stored height is its south-west corner's)."""
    t = W.t.get((int(math.floor(x)), int(math.floor(y))))
    return t[0] if t and t[0] is not None else 0


def project(cam, target_h, size=(1280, 900), focal=None):
    """The camera of arena_view / authored_scene_snapshot: world tile coords + height (negative up) -> pixel."""
    wx, wy, yaw, dist, elev, up = cam
    f = focal or 1000 * size[0] / 1280
    a = yaw * math.tau / 16384
    tx, ty, tz = wx * 128, target_h - up, wy * 128
    px, py, pz = tx + math.sin(a) * dist, ty - elev, tz - math.cos(a) * dist
    fx, fy, fz = tx - px, ty - py, tz - pz
    n = math.sqrt(fx * fx + fy * fy + fz * fz); fx, fy, fz = fx / n, fy / n, fz / n
    # right = forward x down-axis (y is down): for yaw 0 the camera looks north (+z) and right is east (+x)
    rx, ry, rz = fz, 0.0, -fx
    n = math.hypot(rx, rz); rx, rz = rx / n, rz / n
    # screen-down vector = forward x right (so that +y world, which is down, maps to screen down)
    dx, dy, dz = fy * rz - fz * ry, fz * rx - fx * rz, fx * ry - fy * rx

    def P(x, y, h):
        vx, vy, vz = x * 128 - px, h - py, y * 128 - pz
        d = vx * fx + vy * fy + vz * fz
        if d <= 1: return None
        return (size[0] / 2 + f * (vx * rx + vz * rz) / d, size[1] / 2 + f * (vx * dx + vy * dy + vz * dz) / d)
    return P


def outline(im, P, W, S, col=(255, 255, 0), width=2):
    dr = ImageDraw.Draw(im)
    for x, y in S:
        for d, (ddx, ddy) in enumerate(A.DIRS):
            if (x + ddx, y + ddy) in S: continue
            a, b = {0: ((x, y), (x, y + 1)), 2: ((x + 1, y), (x + 1, y + 1)), 1: ((x, y + 1), (x + 1, y + 1)), 3: ((x, y), (x + 1, y))}[d]
            pa, pb = P(a[0], a[1], ground(W, *a)), P(b[0], b[1], ground(W, *b))
            if pa and pb: dr.line([pa, pb], fill=col, width=width)


def bosses(a):
    if a.get('boss'): return [a['boss']]
    return []


def boss_box(a):
    """(x, y, w, d) of the boss's tiles, or None"""
    if a.get('boss_box'): return tuple(a['boss_box'])
    if a.get('boss'): return (a['boss'][1], a['boss'][2], a['size'], a['size'])
    return None


def cams_of(a, W, S):
    B = boss_box(a)
    frame = set(S) | ({(B[0] + i, B[1] + j) for i in range(B[2]) for j in range(B[3])} if B else set())
    cx, cy, span = centre(frame)        # the pictures frame the floor AND the boss (Zuk, the Duke and the Olm stand beyond the edge)
    C = shots.cams(cx, cy, span + 4, a['player'])
    v = a.get('view', 'n')            # the side the boss is on, seen from the player
    yaw = {'n': 0, 'w': 4096, 's': 8192, 'e': 12288}[v]
    px, py = a['player'];
    if a.get('boss'): bx, by = a['boss'][1] + a['size'] / 2, a['boss'][2] + a['size'] / 2
    elif a.get('boss_box'): bx, by = a['boss_box'][0] + a['boss_box'][2] / 2, a['boss_box'][1] + a['boss_box'][3] / 2
    else: bx, by = px, py
    # the played camera looks at the point a third of the way from the player to the boss
    tx, ty = px + .5 + (bx - px - .5) * 0.45, py + .5 + (by - py - .5) * 0.45
    C['played'] = (round(tx, 2), round(ty, 2), yaw) + shots.PLAYED
    return C


def npc_lines(a):
    return [(b[0], b[1], b[2], b[3] if len(b) > 3 else 0) for b in bosses(a)]


def draw(a, which=('top', 'top-empty', 'played', 'threeq', 'check')):
    W, S = floor_of(a); C = cams_of(a, W, S); OUT.mkdir(parents=True, exist_ok=True)
    extra = a.get('locs', [])
    if 'top' in which: shots.real(OUT / f'{a["key"]}-top.png', a['plane'], C['top'], npc_lines(a), extra=extra)
    if 'top-empty' in which: shots.real(OUT / f'{a["key"]}-top-empty.png', a['plane'], C['top'])
    if 'played' in which: shots.real(OUT / f'{a["key"]}-played.png', a['plane'], C['played'], npc_lines(a), extra=extra)
    if 'threeq' in which: shots.real(OUT / f'{a["key"]}-threeq.png', a['plane'], C['threeq'], npc_lines(a), extra=extra)
    if 'check' in which:
        im = Image.open(OUT / f'{a["key"]}-top.png').convert('RGB')
        cam = C['top']; P = project(cam, ground(W, cam[0], cam[1]))
        outline(im, P, W, S)
        for b in bosses(a):
            s = a['size']; B = {(b[1] + i, b[2] + j) for i in range(s) for j in range(s)}
            outline(im, P, W, B, (0, 255, 255), 2)
        p = a['player']; outline(im, P, W, {tuple(p)}, (255, 0, 255), 2)
        ImageDraw.Draw(im).text((8, 8), f'{a["name"]}: floor {len(S)} tiles, bbox {bbox(S)}', fill='white')
        im.save(OUT / f'{a["key"]}-check.png')
    return W, S


if __name__ == '__main__':
    keys = [k for k in sys.argv[1:] if not k.startswith('--')] or [a['key'] for a in ARENAS]
    which = ('check',) if '--overlay' in sys.argv else ('top', 'check') if '--check' in sys.argv else ('top', 'top-empty', 'played', 'threeq', 'check')
    for k in keys:
        try:
            W, S = draw(BY[k], which); print(k, len(S), bbox(S), flush=True)
        except SystemExit as e:
            print(k, 'FAILED', str(e)[:300], flush=True)
