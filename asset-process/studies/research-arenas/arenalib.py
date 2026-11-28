"""arenalib.py: the map as a player meets it, for the arenas study. Read only.

  World(x0, y0, x1, y1, plane)   a window of the real map on one plane (after the bridge shift): per tile the height,
                                 underlay, overlay (id, shape, rotation), terrain flags; every placed object in the window.
  .floor(seed, clip=None, cuts=()) -> set of tiles a player can walk to from `seed` (4-connected, through unblocked edges),
                                 kept inside `clip` (x0, y0, x1, y1) and not through the `cuts` tiles (entrances closed by hand).
  floors()                       underlay / overlay definitions (archive 2 groups 1 and 4; port of floor.rs).
  npcs()                         the NPC census of research-npc (id -> name, size).
Collision is the client's (collision.rs / scene.rs): a wall object (shape 0, 2) blocks tile edges, a diagonal wall (9) and any
standing object (10, 11, and roofs 12..21) with interact type != 0 blocks its footprint, a floor decoration (22) blocks only
with interact type 1, and terrain flag 1 blocks the tile. Objects the SERVER spawns in a fight are not in the map.
"""
import sys, json, pathlib, colorsys, collections
HERE = pathlib.Path(__file__).resolve().parent
TF = HERE.parents[1]
sys.path.insert(0, str(TF / 'studies' / 'research-rooms'))
import rlib as R

DIRS = [(-1, 0), (0, 1), (1, 0), (0, -1)]   # west, north, east, south = wall rotation order


def _i32(v):
    v &= 0xffffffff
    return v - (1 << 32) if v & 0x80000000 else v


def _noise(x, y):
    n = _i32(x + _i32(y * 57)); n = _i32(n ^ _i32(n << 13))
    v = _i32(_i32(_i32(_i32(_i32(n * n) * 15731) + 789221) * n) + 1376312589) & 0x7fffffff
    return (v >> 19) & 255


def _smooth(x, y):
    c = _noise(x - 1, y - 1) + _noise(x + 1, y - 1) + _noise(x - 1, y + 1) + _noise(x + 1, y + 1)
    s = _noise(x - 1, y) + _noise(x + 1, y) + _noise(x, y - 1) + _noise(x, y + 1)
    return c // 16 + s // 8 + _noise(x, y) // 4


def _interp(a, b, f, scale):
    cos = {0: 65536, 256: 46340, 512: 0, 768: -46340}[f * 1024 // scale]
    w = (65536 - cos) >> 1
    return ((a * (65536 - w)) >> 16) + ((b * w) >> 16)


def _interpolated(x, y, scale):
    ix, iy = int(x / scale), int(y / scale); fx, fy = x & (scale - 1), y & (scale - 1)
    a = _interp(_smooth(ix, iy), _smooth(ix + 1, iy), fx, scale)
    b = _interp(_smooth(ix, iy + 1), _smooth(ix + 1, iy + 1), fx, scale)
    return _interp(a, b, fy, scale)


def procedural_height(x, y):
    """Port of terrain.rs procedural_height: the noise height of plane-0 ground that stores no height."""
    x += 932731; y += 556238
    v = _interpolated(x + 45365, y + 91923, 4) - 128 + ((_interpolated(x + 10294, y + 37821, 2) - 128) >> 1) + ((_interpolated(x, y, 1) - 128) >> 2)
    return -8 * max(10, min(60, int(v * 0.3) + 35))


def terrain(gid):
    """Port of terrain.rs Region::decode: {(plane, x, y): [height, underlay, overlay, shape, rot, flags]}; None if absent.
    Heights are the client's (negative = up, 8 units a step; 128 units = one tile)."""
    files = R.CACHE.files(5, gid)
    if 0 not in files or not files[0]: return None
    r = R.Rd(files[0]); T = {}
    bx, by = (gid >> 8) * 64, (gid & 255) * 64
    for plane in range(4):
        for x in range(64):
            for y in range(64):
                below = T[(plane - 1, x, y)][0] if plane else 0
                t = [None, 0, 0, 0, 0, 0]
                while True:
                    op = r.u16()
                    if op == 0:
                        t[0] = procedural_height(bx + x, by + y) if plane == 0 else below - 240
                        break
                    if op == 1:
                        h = r.u8()
                        if h == 1: h = 0
                        t[0] = (below or 0) - 8 * h
                        break
                    if op <= 49:
                        t[2] = r.u16(); t[3] = (op - 2) // 4; t[4] = (op - 2) & 3
                    elif op <= 81: t[5] = op - 49
                    else: t[1] = op - 81
                T[(plane, x, y)] = t
    return T


_FL = None
def floors():
    """({underlay id: rgb}, {overlay id: dict(rgb, texture, hide, rgb2)})"""
    global _FL
    if _FL is None:
        U = {}
        for i, b in R.CACHE.files(2, 1).items():
            r = R.Rd(b); rgb = 0
            while True:
                op = r.u8()
                if op == 0: break
                if op == 1: rgb = r.u24()
            U[i] = rgb
        Ov = {}
        for i, b in R.CACHE.files(2, 4).items():
            r = R.Rd(b); d = dict(rgb=0, texture=None, hide=True, rgb2=None)
            while True:
                op = r.u8()
                if op == 0: break
                if op == 1: d['rgb'] = r.u24()
                elif op == 2: d['texture'] = r.u8()
                elif op == 5: d['hide'] = False
                elif op == 7: d['rgb2'] = r.u24()
                elif op == 9: r.u8()
            Ov[i] = d
        _FL = (U, Ov)
    return _FL


def rgb3(v): return ((v >> 16) & 255, (v >> 8) & 255, v & 255)


def lightness(rgb):
    r, g, b = [c / 255 for c in rgb]
    return colorsys.rgb_to_hls(r, g, b)[1]


_NPC = None
def npcs():
    global _NPC
    if _NPC is None:
        _NPC = {}
        for f in [TF / 'studies' / 'research-npc' / 'census' / 'npcs.jsonl']:
            for l in f.read_text(encoding='utf-8').splitlines():
                try: d = json.loads(l)
                except Exception: continue
                if 'id' in d: _NPC[d['id']] = d
    return _NPC


class World:
    def __init__(self, x0, y0, x1, y1, plane):
        self.x0, self.y0, self.x1, self.y1, self.plane = x0, y0, x1, y1, plane
        self.t = {}          # (x, y) -> [height, underlay, overlay, shape, rot, flags] of the tile SHOWN on this plane
        self.locs = []       # (id, x, y, shape, rot) on this plane
        for rx in range(x0 >> 6, (x1 >> 6) + 1):
            for ry in range(y0 >> 6, (y1 >> 6) + 1):
                gid = (rx << 8) | ry
                try: T = terrain(gid)
                except KeyError: T = None
                if T is None: continue
                _, _, locs = R.square(gid)
                bridge = lambda x, y: bool(T[(1, x, y)][5] & 2)
                for x in range(64):
                    for y in range(64):
                        wx, wy = rx * 64 + x, ry * 64 + y
                        if not (x0 <= wx <= x1 and y0 <= wy <= y1): continue
                        p = plane + 1 if bridge(x, y) else plane
                        if p <= 3: self.t[(wx, wy)] = T[(p, x, y)]
                for oid, p, x, y, sh, rot in locs:
                    if bridge(x, y): p -= 1
                    wx, wy = rx * 64 + x, ry * 64 + y
                    if p == plane and x0 <= wx <= x1 and y0 <= wy <= y1:
                        self.locs.append((oid, wx, wy, sh, rot))
        self._collide()

    def footprint(self, oid, x, y, sh, rot):
        d = R.resolved(oid) or {}
        sx, sz = d.get('size', [1, 1])
        if sh not in (10, 11) : sx = sz = 1
        if rot & 1: sx, sz = sz, sx
        return [(x + i, y + j) for i in range(sx) for j in range(sz)]

    def _collide(self):
        self.blocked = {}         # tile -> reason: 'terrain' or ('obj', oid)
        self.edges = {}           # (x, y, d) -> oid of the wall on that edge (both sides entered)
        for (x, y), t in self.t.items():
            if t[5] & 1: self.blocked[(x, y)] = 'terrain'
        for oid, x, y, sh, rot in self.locs:
            d = R.resolved(oid) or R.defs().get(oid)
            if d is None: continue
            it = d.get('interact', 2)
            if sh in (0, 2):
                if it == 0: continue
                for e in ([rot] if sh == 0 else [rot, (rot + 1) & 3]):
                    self.edges[(x, y, e)] = oid
                    dx, dy = DIRS[e]; self.edges[(x + dx, y + dy, (e + 2) & 3)] = oid
            elif sh in (1, 3): continue
            elif 4 <= sh <= 8: continue
            elif sh == 22:
                if it == 1: self.blocked[(x, y)] = ('obj', oid)
            else:
                if it != 0:
                    for t in self.footprint(oid, x, y, sh, rot): self.blocked[t] = ('obj', oid)

    def floor(self, seed, clip=None, cuts=()):
        clip = clip or (self.x0, self.y0, self.x1, self.y1)
        cuts = set(map(tuple, cuts))
        ok = lambda t: t in self.t and (self.t[t][1] or self.t[t][2]) and t not in self.blocked and t not in cuts and clip[0] <= t[0] <= clip[2] and clip[1] <= t[1] <= clip[3]
        if not ok(tuple(seed)): raise SystemExit(f'seed {seed} is not walkable: {self.blocked.get(tuple(seed))} {self.t.get(tuple(seed))}')
        q = [tuple(seed)]; S = {tuple(seed)}
        while q:
            x, y = q.pop()
            for d, (dx, dy) in enumerate(DIRS):
                n = (x + dx, y + dy)
                if n in S or (x, y, d) in self.edges or not ok(n): continue
                S.add(n); q.append(n)
        return S

    def kind(self, t):
        """The floor kind of a tile: ('o', overlay id) when an overlay covers it (shape 0 = the whole tile; other shapes are
        part-tile blends and still counted as the overlay), else ('u', underlay id)."""
        v = self.t[t]
        return ('o', v[2]) if v[2] else ('u', v[1])

    def kind_rgb(self, k):
        U, Ov = floors()
        if k[0] == 'u': return rgb3(U.get(k[1] - 1, 0))
        o = Ov.get(k[1] - 1, dict(rgb=0, texture=None, rgb2=None))
        return rgb3(o['rgb2'] if o.get('rgb2') is not None and o['rgb'] in (0, 0xff00ff) else o['rgb'])

    def kind_texture(self, k):
        if k[0] == 'u': return None
        return floors()[1].get(k[1] - 1, {}).get('texture')
