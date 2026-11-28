"""Furnishing measures of one room, the same for a real room (from the map) and for one of ours (from a compose table).

A room here is a dict:
  tiles   [(x, y), ...]                floor tiles (x east, y north)
  doors   [(x, y, d), ...]             boundary edges that are a door or doorway; (x, y) is the inside tile, d 0 W 1 N 2 E 3 S
  objects [dict(name, id, cls, func, x, y, w, d, rot, shape, block, h), ...]
          cls: furniture | light | clutter | decor | light-wall | floor | window | door | stairs | trim | wall
          (x, y) is the south-west tile of the footprint and (w, d) the footprint AFTER rotation.
measures(room) returns a flat dict of numbers (see NAMES for their meaning).
"""
import collections

DIRS = [(-1, 0), (0, 1), (1, 0), (0, -1)]

NAMES = {
    'area': 'floor tiles',
    'n_furn': 'standing furniture pieces (named, on the floor; lights and clutter excluded)',
    'occ_share': 'share of floor tiles under standing furniture and standing lights',
    'per10': 'standing objects (furniture + standing lights + clutter) per 10 floor tiles',
    'furn_per10': 'furniture pieces per 10 floor tiles',
    'wall_share': 'share of furniture whose footprint touches a wall edge',
    'wall_edges': 'wall edges round the floor (door edges excluded)',
    'n_decor': 'named wall decorations facing in (shields, banners, paintings, shelves, torches, windows as decoration)',
    'decor_per10w': 'named wall decorations per 10 wall edges',
    'decor_edge_share': 'share of wall edges carrying a wall decoration (windows not counted)',
    'wall_lined': 'share of wall edges with a standing piece against them',
    'bare_run': 'longest run of wall edges on one straight wall with nothing against or on it (no furniture, decoration, window, door)',
    'n_windows': 'windows in the walls',
    'n_trim': 'unnamed full-height wall dressing (beams, frames, trims) facing in',
    'n_floor_named': 'named floor decorations (rugs, carpets, symbols)',
    'floor_cover': 'share of floor tiles carrying any floor decoration (named or not)',
    'grouped_share': 'share of furniture touching at least one other standing piece (8-neighbour footprint contact)',
    'n_groups': 'groups of two or more touching standing pieces',
    'mean_group': 'mean size of those groups',
    'max_group': 'largest group',
    'sym_axis': 'best mirror symmetry of furniture tiles about the room\'s two axes (0..1)',
    'sym_door': 'mirror symmetry of furniture tiles about the door axis (0..1)',
    'sym_id': 'share of furniture pieces with the same object mirrored about the better room axis',
    'faces_door': '1 if furniture or a named wall decoration stands in the far zone opposite the door',
    'reach1': 'share of furniture reachable (a free side tile) from the door',
    'reach2': 'share of furniture reachable by a path two tiles wide',
    'free_share': 'share of floor tiles free of blocking objects',
    'centre_occ': 'share of the centre zone (middle third each way) under standing objects',
    'centre_floor': 'share of the centre zone carrying floor decoration',
    'corner_use': 'share of the four corners with a standing object within one tile',
    'max_repeat': 'most copies of one furniture object',
    'repeat_share': 'share of furniture pieces that are repeats of another in the room',
    'rows': 'touching pairs of the same furniture object (a row)',
    'variety': 'distinct furniture names',
    'variety_all': 'distinct names of everything placed (furniture, decor, lights, clutter, floor)',
    'n_lights': 'light sources (standing and on the wall)',
    'n_clutter': 'small clutter pieces',
    'clutter_by_furn': 'share of clutter touching furniture',
    'n_stairs': 'stairs, ladders, trapdoors',
}


def footprint(o):
    return [(o['x'] + i, o['y'] + j) for i in range(o['w']) for j in range(o['d'])]


def gap(a, b):
    gx = max(0, b['x'] - (a['x'] + a['w']), a['x'] - (b['x'] + b['w']))
    gy = max(0, b['y'] - (a['y'] + a['d']), a['y'] - (b['y'] + b['d']))
    return max(gx, gy)


def boundary(tiles):
    s = set(tiles)
    return [(x, y, d) for x, y in tiles for d, (dx, dy) in enumerate(DIRS) if (x + dx, y + dy) not in s]


def measures(room):
    tiles = [tuple(t) for t in room['tiles']]
    S = set(tiles)
    area = len(tiles)
    xs = [t[0] for t in tiles]; ys = [t[1] for t in tiles]
    x0, x1, y0, y1 = min(xs), max(xs), min(ys), max(ys)
    doors = {tuple(d) for d in room['doors']}
    bnd = boundary(tiles)
    bset = set(bnd)
    wall_edges = [e for e in bnd if e not in doors]
    O = room['objects']
    furn = [o for o in O if o['cls'] == 'furniture']
    slights = [o for o in O if o['cls'] == 'light']
    clutter = [o for o in O if o['cls'] == 'clutter']
    standing = furn + slights
    decor = [o for o in O if o['cls'] in ('decor', 'light-wall') and o.get('inside', True)]
    windows = [o for o in O if o['cls'] == 'window']
    trim = [o for o in O if o['cls'] == 'trim' and o.get('inside', True)]
    floor = [o for o in O if o['cls'] == 'floor']
    stairs = [o for o in O if o['cls'] == 'stairs']
    m = dict(area=area, w=x1 - x0 + 1, h=y1 - y0 + 1, n_furn=len(furn))
    occ = set()
    for o in standing:
        occ |= {t for t in footprint(o) if t in S}
    m['occ_share'] = len(occ) / area
    m['per10'] = 10 * (len(furn) + len(slights) + len(clutter)) / area
    m['furn_per10'] = 10 * len(furn) / area

    def touches_wall(o):
        return any((x, y, d) in bset for x, y in footprint(o) for d in range(4))
    m['wall_share'] = (sum(touches_wall(o) for o in furn) / len(furn)) if furn else None
    m['wall_edges'] = len(wall_edges)
    m['n_decor'] = len(decor)
    m['decor_per10w'] = 10 * len(decor) / max(1, len(wall_edges))
    dec_edges = set()
    # windows are NOT counted as decoration: in the game's maps a window is a wall piece, mostly unnamed, which this
    # study cannot tell from a plain wall, so counting ours would compare unlike with unlike
    for o in decor:
        e = (o['x'], o['y'], o['rot'] % 4)
        if e in bset: dec_edges.add(e)
        else:
            # outside tile facing in, or diagonal decoration: nearest boundary edge
            near = [b for b in wall_edges if abs(b[0] - o['x']) + abs(b[1] - o['y']) <= 1]
            if near: dec_edges.add(near[0])
    m['decor_edge_share'] = len(dec_edges & set(wall_edges)) / max(1, len(wall_edges))
    # walls lined, and the longest bare stretch of one straight wall (no furniture against it, no decoration, no window, no door)
    lined = set()
    for o in standing + clutter:
        for x, y in footprint(o):
            for d in range(4):
                if (x, y, d) in bset: lined.add((x, y, d))
    m['wall_lined'] = len(lined & set(wall_edges)) / max(1, len(wall_edges))
    dressed = lined | dec_edges | doors
    lines = collections.defaultdict(list)
    for (x, y, d) in bnd:
        lines[(d, x if d in (0, 2) else y)].append(((y if d in (0, 2) else x), (x, y, d)))
    best_run = 0
    for k, es in lines.items():
        es.sort(); run = 0; prev = None
        for pos, e in es:
            if prev is not None and pos != prev + 1: run = 0
            run = 0 if e in dressed else run + 1
            best_run = max(best_run, run); prev = pos
    m['bare_run'] = best_run
    m['n_windows'] = len(windows)
    m['n_trim'] = len(trim)
    m['n_floor_named'] = sum(1 for o in floor if o['name'] not in ('null', ''))
    ftiles = set()
    for o in floor: ftiles |= set(footprint(o))
    m['floor_cover'] = len(ftiles & S) / area
    # grouping
    adj = collections.defaultdict(set)
    for i, a in enumerate(standing):
        for j, b in enumerate(standing):
            if i < j and gap(a, b) == 0:
                adj[i].add(j); adj[j].add(i)
    nf = len(furn)
    m['grouped_share'] = (sum(1 for i in range(nf) if adj[i]) / nf) if nf else None
    seen = set(); groups = []
    for i in range(len(standing)):
        if i in seen or not adj[i]: continue
        st = [i]; seen.add(i); g = 0
        while st:
            k = st.pop(); g += 1
            for n in adj[k]:
                if n not in seen: seen.add(n); st.append(n)
        groups.append(g)
    m['n_groups'] = len(groups)
    m['mean_group'] = sum(groups) / len(groups) if groups else 0
    m['max_group'] = max(groups) if groups else 0
    m['pairs'] = sorted({tuple(sorted((standing[i]['func'], standing[j]['func']))) for i in adj for j in adj[i] if i < j}) if standing else []
    m['pair_list'] = [tuple(sorted((standing[i]['func'], standing[j]['func']))) for i in adj for j in adj[i] if i < j]
    # symmetry
    focc = set()
    for o in furn: focc |= {t for t in footprint(o) if t in S}
    def mir(axis, t):
        return (x0 + x1 - t[0], t[1]) if axis == 'x' else (t[0], y0 + y1 - t[1])
    def sym(axis):
        return (sum(1 for t in focc if mir(axis, t) in focc) / len(focc)) if focc else None
    sx, sy = sym('x'), sym('y')
    m['sym_axis'] = max(sx, sy) if focc else None
    best = 'x' if (sx or 0) >= (sy or 0) else 'y'
    # door axis: a door in a west/east wall mirrors north-south about the door's row
    d0 = sorted(doors)[0] if doors else None
    if d0 and focc:
        dx, dy, dd = d0
        if dd in (0, 2):
            m['sym_door'] = sum(1 for t in focc if (t[0], 2 * dy - t[1]) in focc) / len(focc)
        else:
            m['sym_door'] = sum(1 for t in focc if (2 * dx - t[0], t[1]) in focc) / len(focc)
    else:
        m['sym_door'] = None
    if furn:
        pos = {(o['name'], o['x'], o['y'], o['w'], o['d']) for o in furn}
        def mo(o):
            if best == 'x': return (o['name'], x0 + x1 - (o['x'] + o['w'] - 1), o['y'], o['w'], o['d'])
            return (o['name'], o['x'], y0 + y1 - (o['y'] + o['d'] - 1), o['w'], o['d'])
        m['sym_id'] = sum(1 for o in furn if mo(o) in pos) / len(furn)
    else:
        m['sym_id'] = None
    # opposite the door
    m['faces_door'] = None; m['opposite'] = None
    if d0:
        dx, dy, dd = d0
        nx, ny = -DIRS[dd][0], -DIRS[dd][1]        # inward normal
        # far zone: tiles whose depth from the door wall is in the last third, lateral offset within 1 (2 for big rooms)
        lat = 1 if max(m['w'], m['h']) < 9 else 2
        depth = lambda t: (t[0] - dx) * nx + (t[1] - dy) * ny
        side = lambda t: abs((t[0] - dx) * ny + (t[1] - dy) * nx)
        maxd = max(depth(t) for t in tiles if side(t) <= lat)
        # the far zone: the last two rows of floor on the door's line (three tiles wide, five in rooms of 9 or more)
        zone = {t for t in tiles if depth(t) >= maxd - 1 and side(t) <= lat}
        found = [o for o in standing if set(footprint(o)) & zone]
        found += [o for o in decor if (o['x'], o['y']) in zone]
        m['faces_door'] = 1 if found else 0
        m['opposite'] = sorted({o['func'] if o['cls'] == 'furniture' else o['cls'] for o in found})
    # paths
    blockt = set()
    for o in standing:
        if o.get('block', True): blockt |= set(footprint(o))
    free = S - blockt
    m['free_share'] = len(free) / area
    starts = [(x, y) for x, y, d in doors if (x, y) in free]
    def bfs(ok):
        # a doorway is one tile wide by nature: the two-tile path starts on any wide tile within one step of the entry tile
        q = [s for s in starts if s in ok] or [(s[0] + dx, s[1] + dy) for s in starts for dx, dy in DIRS if (s[0] + dx, s[1] + dy) in ok]
        vis = set(q)
        while q:
            x, y = q.pop()
            for dx, dy in DIRS:
                n = (x + dx, y + dy)
                if n in ok and n not in vis: vis.add(n); q.append(n)
        return vis
    wide = {t for t in free if any(all((t[0] + a + i, t[1] + b + j) in free for i in (0, 1) for j in (0, 1)) for a in (-1, 0) for b in (-1, 0))}
    r1, r2 = bfs(free), bfs(wide)
    def reach(o, vis):
        fp = set(footprint(o))
        return any((x + dx, y + dy) in vis for x, y in fp for dx, dy in DIRS)
    m['reach1'] = (sum(reach(o, r1) for o in furn) / nf) if nf and starts else None
    m['reach2'] = (sum(reach(o, r2) for o in furn) / nf) if nf and starts else None
    # centre and corners
    cx0, cx1 = x0 + (x1 - x0 + 1) / 3, x1 - (x1 - x0 + 1) / 3
    cy0, cy1 = y0 + (y1 - y0 + 1) / 3, y1 - (y1 - y0 + 1) / 3
    centre = {t for t in tiles if cx0 - 0.5 <= t[0] <= cx1 + 0.5 and cy0 - 0.5 <= t[1] <= cy1 + 0.5}
    sall = set(occ)
    for o in clutter: sall |= set(footprint(o))
    m['centre_occ'] = len(centre & sall) / len(centre) if centre else 0
    m['centre_floor'] = len(centre & ftiles) / len(centre) if centre else 0
    corners = []
    for cx, cy in ((x0, y0), (x0, y1), (x1, y0), (x1, y1)):
        t = min(tiles, key=lambda t: abs(t[0] - cx) + abs(t[1] - cy))
        corners.append(any(abs(u[0] - t[0]) <= 1 and abs(u[1] - t[1]) <= 1 for u in sall))
    m['corner_use'] = sum(corners) / 4
    # repetition, variety
    cnt = collections.Counter(o['name'] for o in furn)
    m['max_repeat'] = max(cnt.values()) if cnt else 0
    m['repeat_share'] = (nf - len(cnt)) / nf if nf else None
    m['rows'] = sum(1 for i, a in enumerate(furn) for b in furn[i + 1:] if a['name'] == b['name'] and gap(a, b) == 0)
    m['variety'] = len(cnt)
    m['variety_all'] = len({o['name'] for o in furn + slights + clutter + decor + floor if o['name'] not in ('null', '')})
    m['n_lights'] = len(slights) + sum(1 for o in decor if o['cls'] == 'light-wall')
    m['n_clutter'] = len(clutter)
    m['clutter_by_furn'] = (sum(1 for c in clutter if any(gap(c, f) == 0 for f in furn)) / len(clutter)) if clutter else None
    m['n_stairs'] = len(stairs)
    return m
