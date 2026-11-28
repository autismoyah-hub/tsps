"""arenacheck.py: put one of OUR arenas (or a real one) beside the real boss arenas, measure by measure. A guard, not a judge.

usage: arenacheck.py PLACEMENTS.csv FLOOR.json [--json OUT.json] [--quiet]
       arenacheck.py --real KEY                 the same report for a real arena of arenas.py (KEY: nex, tekton, nightmare, ..)

PLACEMENTS.csv  the project's compose table (as roomcheck.py reads it): columns source, tile_x, tile_z, rotation, shape,
                width, depth (UNROTATED) and optionally name, cls, func. What each row is:
                  shape 22   a floor piece: the tile is floor, its KIND read from FLOOR.json "kinds"; a shape-22 row whose
                             source matches "decor" is a floor decoration lying on the floor (a rug, a mark) instead;
                             sources matching "not_floor" are cut tiles under a diagonal wall (not floor)
                  shape 0/2  a wall piece on tile edge `rotation` (0 west, 1 north, 2 east, 3 south; shape 2 = that edge
                             and the next); sources matching "wall_tile" (diagonal / splayed walls) make their tile a wall
                  shape 4-8  a piece hung on a wall
                  shape 10/11 a standing piece; it blocks its tiles unless cls is 'clutter' or 'walk'
                rows whose source starts with 'figure:' or 'boss:' are ignored (the boss comes from FLOOR.json).
FLOOR.json      {"boss": [x, z, w, d],                the boss's tiles (south-west tile, size)
                 "entries": [[x, z], ..],             tiles where players come in
                 "kinds": [[regex, name], ..],        floor kind of a floor piece by its source (first match; default: the file stem)
                 "decor": regex, "not_floor": regex, "wall_tile": regex,          (optional)
                 "pictures": {"top": png, "empty": png, "cam": [x, z, yaw, dist, elev, up]}}   (optional: the straight-down view
                             with and without the boss, from the same camera; without them the palette rules are not measurable)
Prints every measure beside the real band (p10 .. p25 / median / p75 .. p90 of the measured real arenas, data/bands.json),
then every rule of ARENAS.md as `holds`, `OUTSIDE` (with the numbers) or `n/a`. It never prints PASS: an arena inside every
band is merely not unusual; the owner judges it beside real ones.
"""
import sys, csv, json, re, argparse, pathlib, collections
sys.dont_write_bytecode = True
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import arenamodel as AM, rules as RU

SHOW = ['area', 'w', 'h', 'short_side', 'aspect', 'box_fill', 'disc_iou', 'convexity', 'sym_best', 'clear_square', 'open_share',
        'boss_side', 'tiles_per_boss_tile', 'span_per_boss', 'clear_square_per_boss', 'boss_off_centre', 'boss_axis_off', 'boss_to_edge', 'n_entries', 'entry_to_boss',
        'floor_kinds', 'floor_kinds_95', 'plain_share', 'marked_share', 'centre_marked', 'border_marked', 'ring_changes', 'mark_sym', 'flat_share', 'height_total',
        'floor_decor_per100', 'floor_decor_cover',
        'edges', 'edge_wall', 'edge_object', 'edge_void', 'edge_other_ground', 'edge_set_back', 'edge_unseen', 'edge_seen', 'edge_objects_per10', 'edge_kinds', 'edge_kinds_80',
        'edge_top_share', 'edge_repeat', 'edge_height_med', 'edge_height_p90', 'edge_height_max', 'edge_tall_per10', 'edge_decor_per10', 'edge_animated', 'edge_lights', 'beyond_objects_per10', 'beyond_ground_share',
        'inside_objects', 'inside_per100', 'blocked_by_objects', 'blocked_other', 'inside_centre', 'inside_middle', 'inside_band',
        'pic_floor_luma', 'pic_floor_sat', 'pic_floor_plain_share', 'pic_floor_colours_90', 'pic_boss_luma', 'pic_boss_sat', 'pic_luma_contrast', 'pic_abs_luma_contrast',
        'pic_deltaE_mean', 'pic_boss_far_share', 'pic_surround_minus_floor_luma', 'pic_centre_vs_middle_dE', 'pic_band_vs_middle_dE', 'pic_accent_centre', 'pic_accent_surround']


def model_height(path):
    try:
        sys.path.insert(0, str(HERE.parents[1] / 'tools'))
        import osrscache as O
        v = O.decode_model(pathlib.Path(path).read_bytes())['vertices']
        return max(-p[1] for p in v)
    except Exception:
        return 0


def load(csv_path, floor):
    rows = list(csv.DictReader(open(csv_path, newline='')))
    kinds = [(re.compile(p, re.I), n) for p, n in floor.get('kinds', [])]
    rx = lambda k, d: re.compile(floor.get(k, d), re.I)
    decor, notfloor, walltile = rx('decor', r'rug|mark|inlay-on|decal'), rx('not_floor', r'flag-cut|mitre'), rx('wall_tile', r'splay')
    light = re.compile(r'torch|brazier|candle|lamp|lantern|fire|sconce', re.I)
    tiles = {}; wall = {}; cause = {}; objs = []; hcache = {}
    for r in rows:
        src = r['source']; x, z = int(r['tile_x']), int(r['tile_z']); rot = int(r['rotation']); sh = int(r['shape'])
        if src.startswith(('figure:', 'boss:')): continue
        stem = re.split(r'[/\\]', src)[-1].rsplit('.', 1)[0]
        w, d = int(r.get('width') or 1), int(r.get('depth') or 1)
        if sh in (10, 11) and rot % 2: w, d = d, w
        if sh not in (10, 11): w = d = 1
        cls = (r.get('cls') or '').strip(); name = (r.get('name') or '').strip() or stem
        if sh == 22:
            if notfloor.search(src): cause[(x, z)] = ('obj', 'cut tile'); continue
            if decor.search(src) or cls == 'floor':
                objs.append(dict(id=stem, name=name, x=x, y=z, w=1, d=1, shape=22, rot=rot, block=False, h=0, anim=False, light=False)); continue
            tiles[(x, z)] = next((n for p, n in kinds if p.search(src)), re.sub(r'[-_]?\d+$', '', stem)); continue
        if sh in (0, 2):
            if walltile.search(src): cause[(x, z)] = ('obj', stem)
            else:
                for e in ([rot] if sh == 0 else [rot, (rot + 1) & 3]):
                    wall[(x, z, e)] = stem; dx, dz = AM.DIRS[e]; wall[(x + dx, z + dz, (e + 2) & 3)] = stem
        if src not in hcache: hcache[src] = model_height(src)
        block = sh in (10, 11) and cls not in ('clutter', 'walk')
        objs.append(dict(id=re.sub(r'[-_][ab]$', '', stem), name=name, x=x, y=z, w=w, d=d, shape=sh, rot=rot, block=block, h=hcache[src], anim=False,
                         light=bool(light.search(src + ' ' + name))))
    blocked = {}
    for o in objs:
        if o['block']:
            for t in AM.fp(o): blocked[t] = ('obj', o['id'])
    for t, c in cause.items(): blocked.setdefault(t, c); tiles.pop(t, None)
    ent = [tuple(e) for e in floor.get('entries', [])]
    b = floor.get('boss')
    seeds = [e for e in ent if e in tiles and e not in blocked] or [t for t in tiles if t not in blocked][:1]
    S = set(seeds); q = list(seeds)
    while q:
        x, z = q.pop()
        for dd, (dx, dz) in enumerate(AM.DIRS):
            n = (x + dx, z + dz)
            if n in S or n not in tiles or n in blocked or (x, z, dd) in wall: continue
            S.add(n); q.append(n)
    look = {t: dict(kind=k, h=0, void=False) for t, k in tiles.items()}
    cz = dict(blocked)
    M = dict(floor=S, kind={t: tiles[t] for t in S}, hfun=None, wall=wall, cause=cz, look=look, objects=objs,
             boss=tuple(b) if b else None, entries=ent)
    return M


def fmt(v):
    if v is None: return '-'
    if isinstance(v, str): return v
    return str(int(v)) if float(v).is_integer() else f'{v:.2f}'


def report(name, d, B, n, quiet=False, extra=None):
    out = [f'{name}', f'beside {n} real boss arenas (data/bands.json); a guard, not a judge', '']
    if extra: out += extra + ['']
    if not quiet:
        out.append('measure'.ljust(30) + 'this'.rjust(9) + '   real p10 .. p25 / median / p75 .. p90')
        for k in SHOW:
            if k not in d and k not in B: continue
            b = B.get(k); v = d.get(k)
            flag = ''
            if b and v is not None: flag = '  low' if v < b['p10'] - 1e-9 else '  high' if v > b['p90'] + 1e-9 else ''
            out.append(k.ljust(30) + fmt(v).rjust(9) + ('   ' + f"{fmt(b['p10'])} .. {fmt(b['p25'])} / {fmt(b['med'])} / {fmt(b['p75'])} .. {fmt(b['p90'])}" if b else '') + flag)
        out.append('')
    res = RU.evaluate(d, B); hold = 0; outside = []; na = []
    for rid, state, text, rows in res:
        nums = '; '.join(f"{k} {fmt(v)} (limit {fmt(lo)}..{fmt(hi)}; real {fmt(p25)} / {fmt(med)} / {fmt(p75)})" for k, v, lo, hi, p25, med, p75, ok in rows if v is not None)
        out.append(f"{state.ljust(8)}{rid.ljust(36)}{nums or 'not measurable here'}")
        if state == 'holds': hold += 1
        elif state == 'OUTSIDE': outside.append(rid)
        else: na.append(rid)
    out.append('')
    out.append(f'{hold} of {hold + len(outside)} measurable rules hold' + (f'; outside: {", ".join(outside)}' if outside else '') + (f'; not measurable: {", ".join(na)}' if na else ''))
    return '\n'.join(out), res


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('csv', nargs='?'); ap.add_argument('floor', nargs='?'); ap.add_argument('--real'); ap.add_argument('--json'); ap.add_argument('--quiet', action='store_true')
    a = ap.parse_args()
    bb = RU.load_bands(); B = bb['bands']
    if a.real:
        D = json.loads((HERE / 'data' / 'arenas.json').read_text())
        if a.real not in D: raise SystemExit(f'no real arena "{a.real}" in data/arenas.json: {", ".join(D)}')
        m = D[a.real]; d = RU.flat(m); name = f'REAL: {m["name"]} (floor {m["area"]} tiles, {m["w"]} x {m["h"]}, {m["outline"]}; boss {m.get("boss_place")})'
        text, res = report(name, d, B, bb['n'], a.quiet)
    else:
        if not a.csv or not a.floor: ap.error('PLACEMENTS.csv FLOOR.json, or --real KEY')
        floor = json.loads(pathlib.Path(a.floor).read_text(encoding='utf-8'))
        M = load(a.csv, floor)
        m = AM.measure(M)
        pic = floor.get('pictures')
        if pic:
            import draw, palette
            H, D_, c, Z = AM.zones(M)
            base = pathlib.Path(a.floor).parent
            P = draw.project(tuple(pic['cam']), 0)
            m['pic'] = palette.measure(base / pic['top'], base / pic['empty'], P, M, Z, lambda x, y: 0)
        d = RU.flat(m)
        extra = [f"floor {m['area']} tiles, {m['w']} x {m['h']}, outline {m['outline']}; boss {m.get('boss_place')}; edge mostly {m['edge_main']}",
                 'floor kinds: ' + ', '.join(f'{k} {v}' for k, v in m['kinds']),
                 'at the edge: ' + ', '.join(m['edge_named'][:12]), 'inside: ' + (', '.join(m['inside_named']) or 'nothing')]
        text, res = report(f'{a.csv}', d, B, bb['n'], a.quiet, extra)
    print(text)
    if a.json:
        pathlib.Path(a.json).write_text(json.dumps(dict(measures=d, rules=[(rid, st) for rid, st, *_ in res]), indent=1, default=str))


if __name__ == '__main__':
    main()
