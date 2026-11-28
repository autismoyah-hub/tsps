"""Find enclosed interior rooms in every map square and list what is placed in them.

usage: extract.py [OUT.json]      (default rooms-all.json)

Method (per map square, per plane after the bridge shift):
 1. Wall edges: every object placed with a wall shape blocks tile edges as the client's collision does:
    shape 0 rotation r blocks edge r of its tile (0 west, 1 north, 2 east, 3 south); shape 2 (an L corner) blocks r and r+1;
    shape 9 (a diagonal wall) makes its whole tile a wall tile. Shapes 1 and 3 (corner posts) block no edge.
    Only objects whose definition blocks movement (interact type not 0) count, so painted / decorative walls do not.
 2. Doorways without a door object: an unwalled edge whose two neighbours along the same line are walled (gap of one),
    or a gap of two between walls, is closed for the flood and recorded as an opening.
 3. Flood fill over tiles across unblocked edges. A region is a room candidate if it does not reach the square's edge,
    has 20 to 150 tiles, a bounding box 4 to 13 tiles a side, fills at least 55% of its bounding box, and has at least
    one door object or opening on its boundary.
 4. Interior evidence: share of its tiles carrying the terrain flag 4 (the client hides roofs above such tiles) and
    whether roof pieces (shapes 12 to 21) stand on its tiles on the plane above or the same plane. Kept if either holds,
    or if it lies in the instanced/underground band (y >= 4160 tiles, many interiors live there).
Everything placed on the room's tiles (and wall decorations on the outside tiles of its walls that face in) is listed.
"""
import sys, json, collections, pathlib
sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import rlib as R

DIRS = [(-1, 0), (0, 1), (1, 0), (0, -1)]   # west, north, east, south (rotation order)


def blocks_edges(shape, rot):
    if shape == 0: return [rot]
    if shape == 2: return [rot, (rot + 1) % 4]
    return []


def edge_key(x, y, d):
    """Canonical key of the edge d of tile (x, y): ('v', X, y) vertical line at X, or ('h', x, Y)."""
    if d == 0: return ('v', x, y)
    if d == 2: return ('v', x + 1, y)
    if d == 1: return ('h', x, y + 1)
    return ('h', x, y)


def analyse_square(gid):
    flags, over, locs = R.square(gid)
    if flags is None: return []
    D = R.defs()
    rx, ry = gid >> 8, gid & 255
    # bridge shift
    per_plane = collections.defaultdict(list)
    for (oid, p, x, y, sh, rot) in locs:
        if p > 0 and flags[1 * 4096 + x * 64 + y] & 2: p -= 1
        per_plane[p].append((oid, x, y, sh, rot))
    rooms = []
    for plane, L in per_plane.items():
        walls = set(); wall_tiles = set(); roofs_here = set()
        for oid, x, y, sh, rot in L:
            d = R.resolved(oid)
            if d is None: continue
            if sh in (0, 2, 9) and d.get('interact', 2) != 0:
                if sh == 9: wall_tiles.add((x, y))
                for e in blocks_edges(sh, rot): walls.add(edge_key(x, y, e))
            if sh in R.ROOF_SHAPES: roofs_here.add((x, y))
        roofs_above = set()
        for oid, x, y, sh, rot in per_plane.get(plane + 1, []):
            if sh in R.ROOF_SHAPES: roofs_above.add((x, y))
        # openings
        def walled(k): return k in walls
        openings = set()
        for (o, a, b) in list(walls):
            pass
        def is_opening(k):
            if k in walls: return False
            o, a, b = k
            if o == 'v':
                n1, n2 = ('v', a, b - 1), ('v', a, b + 1)
                g2 = [(('v', a, b - 1), ('v', a, b + 1), ('v', a, b + 2)), (('v', a, b - 2), ('v', a, b - 1), ('v', a, b + 1))]
            else:
                n1, n2 = ('h', a - 1, b), ('h', a + 1, b)
                g2 = [(('h', a - 1, b), ('h', a + 1, b), ('h', a + 2, b)), (('h', a - 2, b), ('h', a - 1, b), ('h', a + 1, b))]
            if walled(n1) and walled(n2): return True
            for w1, mid, w2 in g2:
                if walled(w1) and not walled(mid) and walled(w2): return True
            return False
        seen = set()
        for sx in range(64):
            for sy in range(64):
                if (sx, sy) in seen or (sx, sy) in wall_tiles: continue
                region = []; q = [(sx, sy)]; seen.add((sx, sy)); leak = False; ops = set()
                while q:
                    x, y = q.pop()
                    region.append((x, y))
                    if len(region) > 200: leak = True
                    for d, (dx, dy) in enumerate(DIRS):
                        k = edge_key(x, y, d)
                        if k in walls: continue
                        nx, ny = x + dx, y + dy
                        if not (0 <= nx < 64 and 0 <= ny < 64): leak = True; continue
                        if is_opening(k): ops.add(k); continue
                        if (nx, ny) in wall_tiles: continue
                        if (nx, ny) not in seen:
                            seen.add((nx, ny)); q.append((nx, ny))
                if leak or not (20 <= len(region) <= 150): continue
                xs = [t[0] for t in region]; ys = [t[1] for t in region]
                w, h = max(xs) - min(xs) + 1, max(ys) - min(ys) + 1
                if not (4 <= w <= 13 and 4 <= h <= 13): continue
                if len(region) < 0.55 * w * h: continue
                rset = set(region)
                # boundary edges
                bound = []
                for x, y in region:
                    for d, (dx, dy) in enumerate(DIRS):
                        if (x + dx, y + dy) not in rset: bound.append((x, y, d))
                # doors on boundary
                items = []
                doors = 0
                for oid, x, y, sh, rot in L:
                    k = R.kind(oid, sh)
                    inside = (x, y) in rset
                    if sh in R.WALL_SHAPES:
                        es = [edge_key(x, y, e) for e in blocks_edges(sh, rot)] if sh in (0, 2) else []
                        on_bound = any(edge_key(bx, by, bd) in es for bx, by, bd in bound) if es else False
                        if sh == 9:
                            on_bound = any(abs(x - bx) <= 1 and abs(y - by) <= 1 for bx, by in region)
                        if on_bound or (inside and sh in (1, 3)):
                            items.append((oid, x, y, sh, rot, k))
                            if k == 'door': doors += 1
                    elif sh in R.DECOR_SHAPES:
                        # decoration sits on edge rot of its tile; it belongs to the room if that edge is a boundary
                        # edge of the room (whichever side its tile is on), or its tile is inside
                        ek = edge_key(x, y, rot % 4) if sh in (4, 5) else None
                        bset = {edge_key(bx, by, bd) for bx, by, bd in bound}
                        if inside or (ek in bset):
                            side = 'in' if inside else 'out'
                            items.append((oid, x, y, sh, rot, k + ('' if inside else ':outer')))
                    elif sh in R.ROOF_SHAPES:
                        continue
                    else:
                        if inside: items.append((oid, x, y, sh, rot, k))
                if doors == 0 and not ops: continue
                roofflag = sum(1 for x, y in region if flags[plane * 4096 + x * 64 + y] & 4) / len(region)
                roofed = sum(1 for t in region if t in roofs_above or t in roofs_here) / len(region)
                wy = ry * 64
                if roofflag < 0.5 and roofed < 0.5 and wy < 4160: continue
                rooms.append(dict(square=gid, rx=rx, ry=ry, plane=plane, base=[rx * 64, ry * 64], tiles=sorted(region),
                                  w=w, h=h, area=len(region), doors=doors, openings=[list(o) for o in sorted(ops)],
                                  roofflag=round(roofflag, 2), roofed=round(roofed, 2),
                                  items=[list(i) for i in items]))
    return rooms


def main():
    out = pathlib.Path(sys.argv[1] if len(sys.argv) > 1 else R.HERE / 'rooms-all.json')
    allr = []
    sq = R.squares()
    for n, g in enumerate(sq):
        try:
            allr += analyse_square(g)
        except Exception as e:
            print('square', g, 'failed', e)
        if n % 200 == 0: print(n, len(sq), len(allr), flush=True)
    out.write_text(json.dumps(allr))
    print('rooms', len(allr), '->', out)


if __name__ == '__main__':
    main()
