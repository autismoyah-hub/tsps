"""Step 1b test object: the E114 lobby barrel rebuilt at real size, welded, for smooth shading.

Rules applied (from the real barrels' meshes): one vertex per position; no bevels for highlights;
edges made by tight rings and a tone step; shading painted with lightness steps inside one hue;
open underneath. Design is ours: two wide iron hoops, a deep recessed head with five boards, a bung.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, math, pathlib
sys.path.insert(0, _AP.fix(r'<ROOT>\tools'))
from author import Mesh, hsl, object_def

N = 16                      # staves
WOOD, WOOD_GAP, WOOD_DEEP = hsl(5, 2, 28), hsl(5, 2, 25), hsl(5, 3, 17)
HEAD, HEAD_GROOVE = hsl(5, 2, 32), hsl(5, 3, 21)
IRON_TOP, IRON, IRON_UNDER = hsl(3, 2, 23), hsl(3, 2, 18), hsl(3, 2, 13)   # the classic barrels' dark hoop brown, in three tones
BUNG = hsl(5, 2, 36)

PROFILE = [(0, 37), (10, 40), (22, 42.5), (50, 45), (78, 42.5), (90, 40), (100, 37.5)]   # y, radius
HOOPS = [(9, 23), (76, 90)]                                                             # y from, to
GAP = 0.04                 # half-angle of the dark line between two staves, radians


def radius_at(y):
    for (y0, r0), (y1, r1) in zip(PROFILE, PROFILE[1:]):
        if y0 <= y <= y1:
            return r0 + (r1 - r0) * (y - y0) / (y1 - y0)
    raise ValueError(y)


def build():
    m = Mesh()
    RADIAL = lambda c: (c[0], 0, c[2])
    UP = lambda c: (0, 1, 0)
    INWARD = lambda c: (-c[0], 0.2, -c[2])
    m.out = RADIAL
    # staves: each stave has two edge vertices per level; the strip between neighbours is the dark joint
    levels = []
    for y, r in PROFILE:
        row = []
        for i in range(N):
            # the staves are not cut to one length: three stand a little short, as hand-made ones do
            if y == PROFILE[-1][0] and i in (2, 7, 13):
                y_i = y - 3
            else:
                y_i = y
            a0 = 2 * math.pi * i / N + GAP
            a1 = 2 * math.pi * (i + 1) / N - GAP
            row.append((m.vert(r * math.cos(a0), y_i, r * math.sin(a0)), m.vert(r * math.cos(a1), y_i, r * math.sin(a1))))
        levels.append(row)
    for lo, hi in zip(levels, levels[1:]):
        for i in range(N):
            j = (i + 1) % N
            m.quad(lo[i][0], lo[i][1], hi[i][1], hi[i][0], WOOD)
            m.quad(lo[i][1], lo[j][0], hi[j][0], hi[i][1], WOOD_GAP)
    # hoops: their own welded bands standing 3 units proud, with painted top, face and underside
    for y0, y1 in HOOPS:
        r0, r1 = radius_at(y0), radius_at(y1)
        a = m.ring(y0, r0 + 0.6, N, math.pi / N)
        b = m.ring(y0 + 2, r0 + 4.0, N, math.pi / N)
        c = m.ring(y1 - 2, r1 + 4.0, N, math.pi / N)
        d = m.ring(y1, r1 + 0.6, N, math.pi / N)
        m.band(a, b, IRON_UNDER); m.band(b, c, IRON); m.band(c, d, IRON_TOP)
    # chime: inside wall of the rim, down to the recessed head
    top = levels[-1]
    rim = [v for pair in top for v in pair]
    rin = 32.5
    inner = []
    for i in range(N):
        a0 = 2 * math.pi * i / N + GAP
        a1 = 2 * math.pi * (i + 1) / N - GAP
        yt = 97 if i in (2, 7, 13) else 100
        inner += [m.vert(rin * math.cos(a0), yt, rin * math.sin(a0)), m.vert(rin * math.cos(a1), yt, rin * math.sin(a1))]
    n2 = len(rim)
    m.out = UP
    for i in range(n2):
        j = (i + 1) % n2
        m.quad(rim[j], rim[i], inner[i], inner[j], WOOD_GAP if i % 2 else WOOD)      # top of the staves
    m.out = INWARD
    head_ring = m.ring(95, rin - 0.5, N, math.pi / N)
    # inside wall: inner ring (32 verts) down to head ring (16 verts): fan pairs
    for i in range(N):
        j = (i + 1) % N
        p0, p1, p2 = inner[2 * i], inner[2 * i + 1], inner[(2 * i + 2) % n2]
        h0, h1 = head_ring[i], head_ring[j]
        # head_ring[i] sits at the middle of stave i
        m.tri(p1, p0, h0, WOOD_DEEP)
        m.quad(p2, p1, h0, h1, WOOD_DEEP)
    m.out = UP
    # head: five boards across, with narrow dark grooves, cut to the circle
    R = rin + 1.5          # boards run just under the chime wall so no gap shows at the polygon's flats
    cuts = [-R, -20, -18, -7, -5, 6, 8, 19, 21, R]
    def chord(x):
        return math.sqrt(max(R * R - x * x, 0.0))
    for k, (x0, x1) in enumerate(zip(cuts, cuts[1:])):
        col = HEAD if k % 2 == 0 else HEAD_GROOVE
        z0, z1 = chord(x0), chord(x1)
        a = m.vert(x0, 95, -z0); b = m.vert(x1, 95, -z1); c = m.vert(x1, 95, z1); d = m.vert(x0, 95, z0)
        m.quad(a, d, c, b, col)
    # bung on the bulge, toward the south (the usual camera side)
    ang = -math.pi * 0.5 + math.pi / N
    rb = radius_at(50)
    cx, cz = math.cos(ang), math.sin(ang)
    tx, tz = -cz, cx
    m.out = lambda c: (c[0] - (rb - 6) * cx, c[1] - 50, c[2] - (rb - 6) * cz)
    base, tip = [], []
    for k in range(6):
        t = 2 * math.pi * k / 6
        u, w = 4.5 * math.cos(t), 4.5 * math.sin(t)
        base.append(m.vert(rb * cx + tx * u - cx * 1.0, 50 + w, rb * cz + tz * u - cz * 1.0))
        tip.append(m.vert((rb + 4) * cx + tx * u * 0.75, 50 + w * 0.75, (rb + 4) * cz + tz * u * 0.75))
    centre = m.vert((rb + 4.5) * cx, 50, (rb + 4.5) * cz)
    for k in range(6):
        j = (k + 1) % 6
        m.quad(base[k], base[j], tip[j], tip[k], HEAD_GROOVE)
        m.tri(tip[k], tip[j], centre, BUNG)
    return m


if __name__ == '__main__':
    m = build()
    out = pathlib.Path(sys.argv[1]) if len(sys.argv) > 1 else pathlib.Path(__file__).parent / 'barrel.bin'
    out.write_bytes(m.encode(priority=4))
    print(len(m.v), 'vertices', len(m.f), 'faces ->', out)
