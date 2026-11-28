"""arenamodel.py: one neutral description of an arena (a real one from the map, or one of ours from a compose table) and
the measures taken on it. measure.py (real arenas) and arenacheck.py (ours) both call measure(M).

The model M (a dict):
  floor     set of (x, y): tiles a player can stand on and walk to
  kind      {tile: key}: the floor kind of each floor tile (real: underlay / overlay id; ours: the floor piece's kind name)
  hfun      function (x, y) -> height of the tile corner (units, negative up); None = flat
  wall      {(x, y, d): id}: blocked tile edges (d 0 W, 1 N, 2 E, 3 S)
  cause     {tile: 'terrain' | 'void' | ('obj', id)}: why a non-floor tile is not floor (tiles near the floor)
  look      {tile: dict(kind=, h=)}: floor kind and height of non-floor tiles near the floor (for the edge classes), optional
  objects   [dict(id, name, x, y, w, d, shape, rot, block, h, anim, light)]: everything placed on or near the floor
            (x, y = south-west tile of the footprint; w, d after rotation; h = model height in units)
  boss      (x, y, w, d): the boss's tiles (south-west tile, size); may lie outside the floor
  entries   [(x, y), ..]: tiles where players come in (may be empty)
"""
import math, collections, statistics

DIRS = [(-1, 0), (0, 1), (1, 0), (0, -1)]
N8 = [(-1, -1), (-1, 0), (-1, 1), (0, -1), (0, 1), (1, -1), (1, 0), (1, 1)]


def fp(o): return [(o['x'] + i, o['y'] + j) for i in range(o['w']) for j in range(o['d'])]


def hull_of(S):
    """floor + every tile enclosed by it (pits, pillars, blocked centres): the arena's outline filled."""
    xs = [t[0] for t in S]; ys = [t[1] for t in S]
    x0, x1, y0, y1 = min(xs) - 1, max(xs) + 1, min(ys) - 1, max(ys) + 1
    out = {(x0, y0)}; q = [(x0, y0)]
    while q:
        x, y = q.pop()
        for dx, dy in DIRS:
            n = (x + dx, y + dy)
            if x0 <= n[0] <= x1 and y0 <= n[1] <= y1 and n not in S and n not in out: out.add(n); q.append(n)
    return {(x, y) for x in range(x0 + 1, x1) for y in range(y0 + 1, y1) if (x, y) not in out}


def convex_area(H):
    pts = sorted({(x + i, y + j) for x, y in H for i in (0, 1) for j in (0, 1)})
    def half(ps):
        h = []
        for p in ps:
            while len(h) >= 2 and (h[-1][0] - h[-2][0]) * (p[1] - h[-2][1]) - (h[-1][1] - h[-2][1]) * (p[0] - h[-2][0]) <= 0: h.pop()
            h.append(p)
        return h
    hull = half(pts)[:-1] + half(pts[::-1])[:-1]
    return abs(sum(hull[i][0] * hull[(i + 1) % len(hull)][1] - hull[(i + 1) % len(hull)][0] * hull[i][1] for i in range(len(hull)))) / 2


def depth(H):
    """{tile: steps from outside}: 1 = a tile on the outline."""
    d = {}; q = collections.deque()
    for t in H:
        if any((t[0] + dx, t[1] + dy) not in H for dx, dy in N8): d[t] = 1; q.append(t)
    while q:
        t = q.popleft()
        for dx, dy in N8:
            n = (t[0] + dx, t[1] + dy)
            if n in H and n not in d: d[n] = d[t] + 1; q.append(n)
    return d


def largest_square(S):
    best = 0; dp = {}
    for x, y in sorted(S):
        v = 1 + min(dp.get((x - 1, y), 0), dp.get((x, y - 1), 0), dp.get((x - 1, y - 1), 0)); dp[(x, y)] = v; best = max(best, v)
    return best


def zones(M):
    """(hull, depth, centroid, zone of every hull tile: 'centre' | 'middle' | 'band')"""
    H = hull_of(M['floor']); D = depth(H)
    cx = sum(t[0] + .5 for t in H) / len(H); cy = sum(t[1] + .5 for t in H) / len(H)
    rin = max(D.values())
    Z = {}
    for t in H:
        r = math.hypot(t[0] + .5 - cx, t[1] + .5 - cy)
        Z[t] = 'band' if D[t] <= 2 else 'centre' if r <= max(1.5, 0.4 * rin) else 'middle'
    return H, D, (cx, cy), Z


def pct(v, p):
    v = sorted(v)
    if not v: return None
    k = (len(v) - 1) * p; f = math.floor(k); c = min(len(v) - 1, f + 1)
    return v[f] + (v[c] - v[f]) * (k - f)


def measure(M):
    S = set(M['floor']); H, D, (cx, cy), Z = zones(M)
    m = {}
    xs = [t[0] for t in H]; ys = [t[1] for t in H]
    x0, x1, y0, y1 = min(xs), max(xs), min(ys), max(ys)
    w, h = x1 - x0 + 1, y1 - y0 + 1
    m['area'] = len(S); m['hull'] = len(H); m['w'] = w; m['h'] = h; m['aspect'] = max(w, h) / min(w, h)
    m['box_fill'] = len(H) / (w * h)
    r = math.sqrt(len(H) / math.pi)
    disc = {(x, y) for x in range(x0 - 2, x1 + 3) for y in range(y0 - 2, y1 + 3) if math.hypot(x + .5 - cx, y + .5 - cy) <= r}
    m['disc_iou'] = len(H & disc) / len(H | disc)
    m['convexity'] = len(H) / convex_area(H)
    bx, by = (x0 + x1) / 2, (y0 + y1) / 2
    sym = lambda f: sum(1 for t in H if f(t) in H) / len(H)
    m['sym_x'] = sym(lambda t: (int(2 * bx - t[0]), t[1])); m['sym_y'] = sym(lambda t: (t[0], int(2 * by - t[1])))
    m['sym_rot'] = sym(lambda t: (int(bx - (t[1] - by)), int(by + (t[0] - bx)))) if w == h else None
    m['sym_best'] = max(m['sym_x'], m['sym_y'])
    # outline class: a rule on the three numbers, printed with them
    if m['box_fill'] >= 0.93: m['outline'] = 'square' if m['aspect'] <= 1.25 else 'rectangle'
    elif m['disc_iou'] >= 0.84 and m['aspect'] <= 1.25: m['outline'] = 'round'
    elif m['convexity'] >= 0.9: m['outline'] = 'cut-corner box' if m['box_fill'] >= 0.8 else 'convex'
    else: m['outline'] = 'irregular'
    m['inradius'] = max(D.values())
    m['clear_square'] = largest_square(S)
    m['open_share'] = len(S) / len(H)                  # share of the outline's inside that is walkable
    # ---- the boss
    b = M.get('boss')
    if b:
        bt = {(b[0] + i, b[1] + j) for i in range(b[2]) for j in range(b[3])}
        s2 = b[2] * b[3]; s = min(b[2], b[3])
        m['boss_side'] = s; m['boss_tiles'] = s2
        m['tiles_per_boss_tile'] = len(S) / s2
        m['span_per_boss'] = min(w, h) / s
        m['clear_square_per_boss'] = m['clear_square'] / s
        bcx, bcy = b[0] + b[2] / 2, b[1] + b[3] / 2
        m['boss_off_centre'] = math.hypot(bcx - cx, bcy - cy) / (min(w, h) / 2)
        m['boss_axis_off'] = min(abs(bcx - (x0 + x1 + 1) / 2), abs(bcy - (y0 + y1 + 1) / 2))    # tiles off the outline's nearer middle line
        m['boss_on_floor'] = len(bt & S) / len(bt)
        inside = [D[t] for t in bt if t in H]
        m['boss_to_edge'] = (min(inside) - 1) if len(inside) == len(bt) else -1      # floor tiles between the boss and the outline; -1 = he stands on or beyond the edge
        m['boss_place'] = 'beyond the edge' if not inside or len(inside) < len(bt) / 2 else 'at the edge' if m['boss_to_edge'] <= 1 else 'centre' if m['boss_off_centre'] <= 0.25 else 'off centre'
        ent = M.get('entries') or []
        if ent:
            dist = [max(0, max(b[0] - e[0], e[0] - (b[0] + b[2] - 1)), max(b[1] - e[1], e[1] - (b[1] + b[3] - 1))) for e in ent]
            m['entry_to_boss'] = min(dist)
        else: m['entry_to_boss'] = None
        m['n_entries'] = len(ent)
    # ---- the floor
    K = M['kind']; cnt = collections.Counter(K[t] for t in S)
    dom = cnt.most_common(1)[0][0]
    m['floor_kinds'] = len(cnt)
    m['plain_share'] = cnt[dom] / len(S)
    acc = 0; n95 = 0
    for k, v in cnt.most_common():
        acc += v; n95 += 1
        if acc >= 0.95 * len(S): break
    m['floor_kinds_95'] = n95
    m['kinds'] = [(str(k), v) for k, v in cnt.most_common(8)]
    zdom = {}
    for z in ('centre', 'middle', 'band'):
        c = collections.Counter(K[t] for t in S if Z[t] == z)
        zdom[z] = c.most_common(1)[0][0] if c else None
    ring1 = collections.Counter(K[t] for t in S if D[t] == 1)
    m['centre_marked'] = int(zdom['centre'] is not None and zdom['centre'] != zdom['middle'])
    m['border_marked'] = int(bool(ring1) and ring1.most_common(1)[0][0] != zdom['middle'])
    rmax = max(math.hypot(t[0] + .5 - cx, t[1] + .5 - cy) for t in S) + 1e-6
    bands = collections.defaultdict(collections.Counter)
    for t in S: bands[min(5, int(6 * math.hypot(t[0] + .5 - cx, t[1] + .5 - cy) / rmax))][K[t]] += 1
    seq = [bands[i].most_common(1)[0][0] for i in sorted(bands)]
    m['ring_changes'] = sum(1 for a, c in zip(seq, seq[1:]) if a != c)
    m['band_purity'] = sum(bands[i].most_common(1)[0][1] for i in bands) / len(S)
    m['concentric_gain'] = m['band_purity'] - m['plain_share']
    # how the non-plain tiles are arranged: mirror symmetry of the marking about the outline's axes
    mark = {t for t in S if K[t] != dom}
    m['marked_share'] = len(mark) / len(S)
    if mark:
        m['mark_sym'] = max(sum(1 for t in mark if (int(2 * bx - t[0]), t[1]) in mark) / len(mark), sum(1 for t in mark if (t[0], int(2 * by - t[1])) in mark) / len(mark))
    else: m['mark_sym'] = None
    hf = M.get('hfun')
    if hf:
        flat = 0; hs = []
        for x, y in S:
            c = [hf(x, y), hf(x + 1, y), hf(x, y + 1), hf(x + 1, y + 1)]
            flat += len(set(c)) == 1; hs.append(sum(c) / 4)
        m['flat_share'] = flat / len(S)
        m['height_range'] = pct(hs, 0.95) - pct(hs, 0.05)
        m['height_total'] = max(hs) - min(hs)
        lv = collections.Counter(round(v / 8) for v in hs); acc = 0; n = 0
        for k, v in lv.most_common():
            acc += v; n += 1
            if acc >= 0.9 * len(hs): break
        m['height_levels_90'] = n
    else:
        m['flat_share'] = 1.0; m['height_range'] = 0; m['height_total'] = 0; m['height_levels_90'] = 1
    O = M['objects']
    fdec = [o for o in O if o['shape'] == 22 and (o['x'], o['y']) in H]
    m['floor_decor_per100'] = 100 * len(fdec) / len(H)
    m['floor_decor_kinds'] = len({o['id'] for o in fdec})
    m['floor_decor_cover'] = len({(o['x'], o['y']) for o in fdec}) / len(H)
    cz = [t for t in H if Z[t] == 'centre']
    m['floor_decor_centre'] = len({(o['x'], o['y']) for o in fdec if Z[(o['x'], o['y'])] == 'centre'}) / max(1, len(cz))
    # ---- the edge
    outer = [(t, d) for t in S for d, (dx, dy) in enumerate(DIRS) if (t[0] + dx, t[1] + dy) not in H]
    inner = [(t, d) for t in S for d, (dx, dy) in enumerate(DIRS) if (t[0] + dx, t[1] + dy) in H and (t[0] + dx, t[1] + dy) not in S]
    m['edges'] = len(outer); m['inner_edges'] = len(inner)
    m['edge_per_sqrt_area'] = len(outer) / math.sqrt(len(H))       # 4.0 = a square, 3.5 = a disc on a grid would be 4 too: jaggedness shows above 4.5
    cause = collections.Counter(); wall_ids = collections.Counter()
    look = M.get('look') or {}
    seen_tiles = set()                       # tiles carrying something modelled (not floor decoration)
    for o in M['objects']:
        if o['shape'] != 22: seen_tiles |= set(fp(o))
    unseen_wall = M.get('wall_unseen') or ()
    for t, d in outer:
        n = (t[0] + DIRS[d][0], t[1] + DIRS[d][1])
        if (t[0], t[1], d) in M['wall'] and (t[0], t[1], d) not in unseen_wall: cause['wall'] += 1; wall_ids[M['wall'][(t[0], t[1], d)]] += 1; continue
        c = M['cause'].get(n, 'void')
        if (t[0], t[1], d) in unseen_wall: c = 'unseen'
        if isinstance(c, tuple): cause['object'] += 1; wall_ids[c[1]] += 1; continue
        if c == 'terrain':
            ln, lt = look.get(n), look.get(t)
            if not ln or ln.get('void'): c = 'void'
            elif lt and abs(ln['h'] - lt['h']) >= 48: c = 'drop' if ln['h'] > lt['h'] else 'rise'
            elif lt and ln['kind'] != lt['kind']: c = 'other ground'
            else: c = 'unseen'
        if c in ('unseen', 'clip'):
            # nothing seen ON the edge: is something modelled standing just behind it (one or two tiles out)? = a set-back edge
            n2 = (n[0] + DIRS[d][0], n[1] + DIRS[d][1])
            if n in seen_tiles or n2 in seen_tiles: c = 'set back'
        cause[c] += 1
    for k in ('wall', 'object', 'void', 'drop', 'rise', 'other ground', 'set back', 'unseen', 'clip'):
        m['edge_' + k.replace(' ', '_')] = cause[k] / max(1, len(outer))
    m['edge_built'] = m['edge_wall'] + m['edge_object']            # bounded by something modelled (walls, rocks, pillars, fences)
    m['edge_main'] = max(('wall', 'object', 'void', 'drop', 'rise', 'other ground', 'set back', 'unseen', 'clip'), key=lambda k: cause[k])
    # edge dressing: everything within two tiles outside the outline, and wall pieces / wall decoration on the outline's own tiles
    near = {(t[0] + i, t[1] + j) for t in H if D[t] == 1 for i in (-2, -1, 0, 1, 2) for j in (-2, -1, 0, 1, 2)} - H
    eobj = []
    for o in O:
        if o['shape'] == 22: continue
        f = set(fp(o))
        if f & near and not f & H: eobj.append(o)
        elif o['shape'] < 10 and any(t in H and D[t] == 1 for t in f): eobj.append(o)
    m['edge_objects'] = len(eobj)
    m['edge_objects_per10'] = 10 * len(eobj) / max(1, len(outer))
    ec = collections.Counter(o['id'] for o in eobj)
    m['edge_kinds'] = len(ec)
    m['edge_top_share'] = (ec.most_common(1)[0][1] / len(eobj)) if eobj else None
    acc = 0; n = 0
    for k, v in ec.most_common():
        acc += v; n += 1
        if acc >= 0.8 * len(eobj): break
    m['edge_kinds_80'] = n
    m['edge_repeat'] = (len(eobj) / len(ec)) if ec else None
    hs = [o['h'] for o in eobj if o.get('h', 0) >= 48]
    m['edge_height_med'] = statistics.median(hs) if hs else 0
    m['edge_height_p90'] = pct(hs, 0.9) if hs else 0
    m['edge_height_max'] = max(hs) if hs else 0
    tall = [o for o in eobj if o.get('h', 0) >= 200 and o['shape'] in (10, 11)]
    m['edge_tall_standing'] = len(tall)                               # pillars, statues, towers standing in the edge
    m['edge_tall_per10'] = 10 * len(tall) / max(1, len(outer))
    m['edge_decor'] = sum(1 for o in eobj if 4 <= o['shape'] <= 8)
    m['edge_decor_per10'] = 10 * m['edge_decor'] / max(1, len(outer))
    m['edge_animated'] = sum(1 for o in eobj if o.get('anim'))
    m['edge_lights'] = sum(1 for o in eobj if o.get('light'))
    m['edge_named'] = sorted({o['name'] for o in eobj if o['name'] not in ('null', '')})[:14]
    # the world beyond the edge: three to six tiles outside the outline
    far = {(t[0] + i, t[1] + j) for t in H if D[t] == 1 for i in range(-6, 7) for j in range(-6, 7)} - H - near
    fobj = [o for o in O if o['shape'] != 22 and set(fp(o)) & far and not set(fp(o)) & (H | near)]
    m['beyond_objects_per10'] = 10 * len(fobj) / max(1, len(outer))
    m['beyond_ground_share'] = sum(1 for t in far if look.get(t) and not look[t].get('void')) / max(1, len(far))
    # ---- inside
    ins = [o for o in O if o['shape'] in (10, 11) and set(fp(o)) & H]
    blk = [o for o in ins if o['block']]
    m['inside_objects'] = len(ins); m['inside_blockers'] = len(blk)
    m['inside_per100'] = 100 * len(ins) / len(H)
    m['blockers_per100'] = 100 * len(blk) / len(H)
    btiles = set()
    for o in blk: btiles |= set(fp(o)) & H
    m['blocked_by_objects'] = len(btiles) / len(H)
    m['blocked_other'] = (len(H) - len(S) - len(btiles - S)) / len(H)       # pits, water, blocked ground inside the outline
    zc = collections.Counter()
    for o in ins:
        f = [t for t in fp(o) if t in H]
        zc[collections.Counter(Z[t] for t in f).most_common(1)[0][0]] += 1
    for z in ('centre', 'middle', 'band'): m['inside_' + z] = zc[z]
    it = set()
    for o in ins: it |= set(fp(o)) & H
    if it:
        m['inside_sym'] = max(sum(1 for t in it if (int(2 * bx - t[0]), t[1]) in it) / len(it), sum(1 for t in it if (t[0], int(2 * by - t[1])) in it) / len(it))
    else: m['inside_sym'] = None
    m['inside_kinds'] = len({o['id'] for o in ins})
    ic = collections.Counter(o['id'] for o in ins)
    m['inside_max_repeat'] = ic.most_common(1)[0][1] if ic else 0
    m['inside_animated'] = sum(1 for o in ins if o.get('anim'))
    m['inside_lights'] = sum(1 for o in ins if o.get('light'))
    m['inside_tall'] = sum(1 for o in ins if o.get('h', 0) >= 200)
    m['inside_named'] = sorted({o['name'] for o in ins if o['name'] not in ('null', '')})[:14]
    m['lights_total'] = m['edge_lights'] + m['inside_lights']
    m['animated_total'] = m['edge_animated'] + m['inside_animated']
    return m
