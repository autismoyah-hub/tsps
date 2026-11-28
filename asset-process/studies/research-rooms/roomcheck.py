"""roomcheck.py: score a composed room against how the game's own rooms of its kind are furnished (ROOMS.md).

usage: roomcheck.py PLACEMENTS.csv --kind KIND [--json OUT.json] [--quiet]
  PLACEMENTS.csv  the compose table as the armoury's compose.py writes it: columns source, tile_x, tile_z, rotation, shape,
                  width, depth (width and depth UNROTATED, as compose.py places them). Optional columns name, cls, func
                  override what is read from the source's file name (see PIECES below). A source 'real:ID' is a real
                  object (its name and size come from the cache); 'figure:ID' rows are ignored.
  KIND            armoury | library | workshop | smithy | kitchen | bedroom | throne | shop | bank | temple | laboratory |
                  tavern | any   (a kind with fewer than 8 study rooms is checked against all 96 study rooms)
The floor is every tile carrying a shape-22 piece; doors are shape-0 rows whose source names a door.
Prints every measure beside the real range for the kind, then each rule as PASS or what is missing, then PASS or FAIL.
Exit status 0 on PASS, 1 on FAIL.
"""
import sys, csv, json, math, re, argparse, pathlib
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import roommeasure as M

# What our pieces are, by file name (first match wins): regex -> (name, class, function)
PIECES = [
    (r'arm-rack', ('Weapon rack', 'furniture', 'rack')), (r'arm-stand', ('Armour stand', 'furniture', 'rack')),
    (r'arm-dummy', ('Training dummy', 'furniture', 'rack')), (r'arm-bench', ('Grindstone bench', 'furniture', 'workstation')),
    (r'arm-chest|docchest|trial-astra-chest|chest', ('Chest', 'furniture', 'chest')), (r'arm-pail', ('Water pail', 'clutter', 'storage')),
    (r'obs1-orrery', ('Orrery', 'furniture', 'machine')), (r'obs1-telescope', ('Telescope', 'furniture', 'machine')),
    (r'obs2-cabinet', ('Instrument cabinet', 'furniture', 'shelf')), (r'obs2-table|maptable', ('Table', 'furniture', 'table')),
    (r'obs3-candle', ('Candle stand', 'light', 'light')), (r'obs3-globe', ('Globe', 'furniture', 'machine')),
    (r'obs3-stool|cal-stool|barstool', ('Stool', 'furniture', 'seat')), (r'obs3-rug', ('Rug', 'floor', 'floor')),
    (r'scribe-desk|[/\\]desk\.bin|trial-astra-table', ('Desk', 'furniture', 'table')), (r'scribe-chair', ('Chair', 'furniture', 'seat')),
    (r'scribe-cabinet|drawers', ('Cabinet', 'furniture', 'shelf')), (r'scribe-bookcase', ('Bookcase', 'furniture', 'bookcase')),
    (r'crate', ('Crate', 'furniture', 'storage')), (r'barrel', ('Barrel', 'furniture', 'storage')),
    (r'bench', ('Bench', 'furniture', 'seat')), (r'lectern', ('Lectern', 'furniture', 'table')),
    # shell and kit
    (r'door', ('Door', 'door', 'door')), (r'window|slit', ('Window', 'window', 'window')),
    (r'wall_banner|banner', ('Banner', 'decor', 'decor')), (r'sconce|torch|bracket|candle', ('Wall light', 'light-wall', 'light')),
    (r'shield|plaque|trophy|tapestry|painting|shelf-wall|wall-shelf|hanging', ('Wall decoration', 'decor', 'decor')),
    (r'hatch|trapdoor|stair|ladder', ('Hatch', 'stairs', 'stairs')),
    (r'rug|ring|dial|carpet|inlay|mat\b|symbol', ('Floor decoration', 'floor', 'floor')),
]
PIECES = [(re.compile(p, re.I), v) for p, v in PIECES]
TECH = re.compile(r'shade|wall_low', re.I)        # grounding patches and cutaway courses: not furnishing

WALL_FUNCS = None   # filled from companions.json: functions that stand against a wall in >= 75% of real cases


def piece(src, shape):
    base = re.split(r'[/\\]', src)[-1]
    for p, v in PIECES:
        if p.search(src):
            return v
    if shape in (0, 1, 2, 3, 9): return ('Wall', 'wall', 'wall')
    if shape == 22: return ('Floor', 'floor-base', 'floor')
    if shape in (4, 5, 6, 7, 8): return (base, 'decor', 'decor')
    return (base, 'furniture', 'other')


def load(path):
    rows = list(csv.DictReader(open(path, newline='')))
    tiles, doors, objs = set(), set(), []
    for r in rows:
        src = r['source']; x, z = int(r['tile_x']), int(r['tile_z']); rot = int(r['rotation']); sh = int(r['shape'])
        if src.startswith('figure:') or TECH.search(src): continue
        w, d = int(r.get('width') or 1), int(r.get('depth') or 1)
        if src.startswith('real:'):
            sys.path.insert(0, str(HERE)); import rlib as RL
            oid = int(src[5:]); dd = RL.resolved(oid)
            name = dd['name']; w, d = dd['size']; func = RL.func_class(name); cls = RL.kind(oid, sh)
        else:
            name, cls, func = piece(src, sh)
            base = re.split(r'[/\\]', src)[-1].rsplit('.', 1)[0]
            if cls in ('furniture', 'light', 'clutter'):
                name = name + ' (' + re.sub(r'-[ab]$', '', base) + ')'
        name, cls, func = r.get('name') or name, r.get('cls') or cls, r.get('func') or func
        if sh in (10, 11) and rot % 2: w, d = d, w
        if sh not in (10, 11): w = d = 1
        if sh == 22 and re.search(r'flag-cut|mitre', src): continue   # a cut tile under a diagonal (splayed) wall: not floor,
        #                                                                as the game's maps make a diagonal wall's tile a wall tile
        if sh == 22: tiles.add((x, z))
        if cls == 'door': doors.add((x, z, rot % 4))
        if cls in ('wall', 'floor-base', 'door'): continue
        objs.append(dict(id=src, name=name, cls=cls, func=func, x=x, y=z, w=w, d=d, rot=rot, shape=sh, block=cls != 'clutter', h=0, inside=True))
    return dict(tiles=sorted(tiles), doors=sorted(doors), objects=objs)


def check(room, kind):
    ranges = json.loads((HERE / 'ranges.json').read_text())
    comp = json.loads((HERE / 'companions.json').read_text())
    wall_funcs = {f for f, v in comp.items() if v['against_wall'] >= 0.75 and f not in ('other',)}
    grp = kind if kind in ranges and ranges[kind]['n'] >= 8 else 'all'
    R = ranges[grp]; A = ranges['all']; W = ranges['well']
    m = M.measures(room)
    area = m['area']
    q = lambda k, p, src=R: src[k][p] if k in src else A[k][p]
    out = []   # (ok, rule id, message)

    def rule(rid, ok, msg): out.append((bool(ok), rid, msg))
    furn = [o for o in room['objects'] if o['cls'] == 'furniture']
    # ---- density: is it sparse?
    lo = math.ceil(q('furn_per10', 'p25') * area / 10); mid = round(q('furn_per10', 'med') * area / 10); hi = math.floor(q('furn_per10', 'p90') * area / 10)
    n = m['n_furn']
    rule('D1 pieces', lo <= n <= hi, f'{n} standing pieces; {kind} rooms of {area} tiles hold {lo} to {hi}, typically {mid}' +
         (f': needs {lo - n} to {hi - n} more' if n < lo else f': {n - hi} too many' if n > hi else ''))
    t = q('occ_share', 'p25')
    rule('D2 floor used', m['occ_share'] >= t, f"furniture covers {m['occ_share']:.2f} of the floor; at least {t:.2f}" +
         (f': about {math.ceil((t - m["occ_share"]) * area)} more tiles under furniture' if m['occ_share'] < t else ''))
    t = q('wall_lined', 'p25')
    need = math.ceil(t * m['wall_edges']) - round(m['wall_lined'] * m['wall_edges'])
    rule('D3 walls lined', m['wall_lined'] >= t, f"{m['wall_lined']:.2f} of wall edges have a piece against them; at least {t:.2f}" +
         (f': {need} more wall tiles need a piece against them' if need > 0 else ''))
    t = q('bare_run', 'p75')
    rule('D4 no bare wall', m['bare_run'] <= t, f"longest bare stretch of one wall {m['bare_run']} tiles; at most {t:.0f}" +
         (': put a piece against it or a decoration on it' if m['bare_run'] > t else ''))
    t = max(q('decor_per10w', 'med'), 1.0)
    need = math.ceil(t * m['wall_edges'] / 10) - m['n_decor']
    rule('D5 wall decoration', m['n_decor'] >= math.ceil(t * m['wall_edges'] / 10),
         f"{m['n_decor']} wall decorations on {m['wall_edges']} wall edges; at least {math.ceil(t * m['wall_edges'] / 10)} "
         f"(well-furnished rooms {W['decor_per10w']['p25']:.1f} to {W['decor_per10w']['p75']:.1f} per 10 edges = "
         f"{math.ceil(W['decor_per10w']['p25'] * m['wall_edges'] / 10)} to {math.floor(W['decor_per10w']['p75'] * m['wall_edges'] / 10)})" +
         (f': needs {need} more wall decorations' if need > 0 else ''))
    t = q('variety_all', 'p25')
    rule('D6 variety', m['variety_all'] >= t, f"{m['variety_all']} distinct things; at least {t:.0f}" +
         (f': {math.ceil(t - m["variety_all"])} more kinds of piece' if m['variety_all'] < t else ''))
    t = q('corner_use', 'p25')
    rule('D7 corners', m['corner_use'] >= t, f"{m['corner_use'] * 4:.0f} of 4 corners used; at least {t * 4:.0f}")
    # ---- placement: is there a reason?
    t = q('wall_share', 'p25')
    rule('P1 against walls', (m['wall_share'] or 0) >= t, f"{(m['wall_share'] or 0):.2f} of furniture stands against a wall; at least {t:.2f}")
    bset = set(M.boundary([tuple(x) for x in room['tiles']]))
    off = [o['name'] for o in room['objects'] if o['cls'] in ('furniture', 'light') and o['func'] in wall_funcs
           and not any((x, y, d) in bset for x, y in M.footprint(o) for d in range(4))]
    wf = [o for o in room['objects'] if o['cls'] in ('furniture', 'light') and o['func'] in wall_funcs]
    rule('P2 wall pieces at walls', len(off) <= 0.25 * len(wf), 'pieces the game stands against a wall (' + ', '.join(sorted(wall_funcs)) + ') ' +
         f'at least 3 in 4 against a wall; ' + ('all are' if not off else f'{len(wf) - len(off)} of {len(wf)} are; NOT against a wall: ' + ', '.join(off)))
    t = q('grouped_share', 'p25')
    rule('P3 groups', (m['grouped_share'] or 0) >= t, f"{(m['grouped_share'] or 0):.2f} of furniture touches another piece; at least {t:.2f}")
    # companions
    st = [o for o in room['objects'] if o['cls'] in ('furniture', 'light')]
    lone = []
    for o in st:
        nb = {b['func'] for b in st if b is not o and M.gap(o, b) == 0}
        nb |= {c['func'] for c in room['objects'] if c['cls'] == 'clutter' and M.gap(o, c) == 0}
        if o['func'] == 'seat' and not nb & {'table', 'workstation', 'throne'}: lone.append(f"{o['name']} should touch a table or desk")
        if o['func'] == 'workstation' and not nb: lone.append(f"{o['name']} should have a companion (storage, seat, table or a second station) beside it")
        if o['func'] == 'bed' and not nb & {'storage', 'chest', 'shelf', 'rack'}: lone.append(f"{o['name']} should have a chest or cabinet beside it")
        if o['func'] == 'throne' and not nb & {'table', 'light', 'throne'}: lone.append(f"{o['name']} should have lights or a table beside it")
    rule('P4 companions', not lone, 'seats at tables, workstations and beds with a companion' + ('' if not lone else ': ' + '; '.join(lone)))
    t = q('max_repeat', 'p25')
    rule('P5 repetition', m['max_repeat'] >= t, f"the most-repeated piece appears {m['max_repeat']} times; {kind} rooms repeat their main piece at least {t:.0f} times")
    rule('P6 faces the door', m['faces_door'] == 1, 'something stands or hangs opposite the door' if m['faces_door'] else 'nothing faces the door: put the room\'s main piece or a decoration on the far wall on the door\'s line')
    cmed, cp75, cp25 = q('centre_occ', 'med'), q('centre_occ', 'p75'), q('centre_occ', 'p25')
    if cmed < 0.2:
        rule('P7 centre', m['centre_occ'] <= cp75 + 1e-9, f"centre {m['centre_occ']:.2f} occupied; {kind} rooms keep the centre clear (at most {cp75:.2f})" +
             (': move the centre piece to a wall or into a group' if m['centre_occ'] > cp75 + 1e-9 else ''))
    else:
        rule('P7 centre', m['centre_occ'] >= cp25, f"centre {m['centre_occ']:.2f} occupied; {kind} rooms put their main group in the centre (at least {cp25:.2f})")
    rule('P8 reach', (m['reach1'] or 0) >= 1 and (m['reach2'] or 0) >= q('reach2', 'p25'),
         f"every piece reachable from the door: {(m['reach1'] or 0):.2f}; by a two-tile path: {(m['reach2'] or 0):.2f} (at least {q('reach2', 'p25'):.2f})")
    lo, hi = q('free_share', 'p10'), q('free_share', 'p90')
    rule('P9 free floor', lo <= m['free_share'] <= hi, f"free floor {m['free_share']:.2f}; real {kind} rooms {lo:.2f} to {hi:.2f}")
    return m, out, grp


def main():
    ap = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    ap.add_argument('csv'); ap.add_argument('--kind', required=True); ap.add_argument('--json'); ap.add_argument('--quiet', action='store_true')
    a = ap.parse_args()
    room = load(a.csv)
    m, out, grp = check(room, a.kind)
    ranges = json.loads((HERE / 'ranges.json').read_text())
    if not a.quiet:
        print(f'{a.csv}\nkind {a.kind} (ranges from study group "{grp}", n={ranges[grp]["n"]}); floor {m["area"]} tiles, {m["w"]} x {m["h"]}')
        print('measure'.ljust(17) + 'this room'.rjust(10) + '   real p25 / median / p75')
        for k in ['n_furn', 'furn_per10', 'per10', 'occ_share', 'wall_share', 'wall_lined', 'bare_run', 'n_decor', 'decor_per10w',
                  'grouped_share', 'mean_group', 'max_repeat', 'variety_all', 'sym_axis', 'sym_id', 'faces_door', 'centre_occ',
                  'centre_floor', 'corner_use', 'reach2', 'free_share', 'n_lights', 'n_clutter', 'floor_cover']:
            v = m.get(k); s = ranges[grp].get(k)
            print(k.ljust(17) + ('-' if v is None else f'{v:.2f}').rjust(10) + ('   ' + f"{s['p25']:.2f} / {s['med']:.2f} / {s['p75']:.2f}" if s else ''))
        print()
    fails = [o for o in out if not o[0]]
    for ok, rid, msg in out:
        print(('PASS ' if ok else 'MISS ') + rid.ljust(26) + msg)
    print('\nPASS' if not fails else f'\nFAIL: {len(fails)} of {len(out)} rules missed')
    if a.json:
        pathlib.Path(a.json).write_text(json.dumps(dict(measures={k: v for k, v in m.items() if k not in ('pair_list',)}, rules=out, group=grp), indent=1, default=str))
    sys.exit(0 if not fails else 1)


if __name__ == '__main__':
    main()
