"""rules.py: the arena rules as data, used by table.py (how many real arenas each holds for) and arenacheck.py (ours).

A rule is (id, what it says, [tests]); a test is (measure, low, high, aim) where low / high are numbers, None, or
'p10' / 'p25' / 'p75' / 'p90' of the measured arenas (data/bands.json). A rule HOLDS when every test whose measure is
present lies inside its limits. `aim` is what to build to (the middle half of the real arenas), printed, not tested.

  rules.py            prints every rule with the number of real arenas it holds for and the ones it does not
"""
import sys, json, pathlib
HERE = pathlib.Path(__file__).resolve().parent


def flat(m):
    """One arena's measures as one flat dict of numbers (picture measures as pic_*, room measures as room_*)."""
    d = {k: v for k, v in m.items() if isinstance(v, (int, float)) and not isinstance(v, bool)}
    p = m.get('pic') or {}
    for k, v in p.items():
        if isinstance(v, (int, float)): d['pic_' + k] = v
        elif isinstance(v, dict):
            for kk, vv in v.items():
                if isinstance(vv, (int, float)): d[f'pic_{k}_{kk}'] = vv
    if p.get('floor') and p.get('surround'): d['pic_surround_minus_floor_luma'] = p['surround']['luma'] - p['floor']['luma']
    if p.get('floor_centre_luma') is not None and p.get('floor_band_luma') is not None: d['pic_centre_minus_band_luma'] = p['floor_centre_luma'] - p['floor_band_luma']
    if p.get('luma_contrast') is not None: d['pic_abs_luma_contrast'] = abs(p['luma_contrast'])
    r = (m.get('room') or {}).get('all', {}).get('measures') or {}
    for k, v in r.items():
        if isinstance(v, (int, float)): d['room_' + k] = v
    if 'w' in d: d['short_side'] = min(d['w'], d['h']); d['long_side'] = max(d['w'], d['h'])
    if 'edge_built' in d: d['edge_seen'] = d['edge_built'] + d.get('edge_void', 0) + d.get('edge_other_ground', 0) + d.get('edge_drop', 0) + d.get('edge_rise', 0) + d.get('edge_set_back', 0)
    if d.get('marked_share', 0) < 0.05: d.pop('mark_sym', None)
    if 'beyond_objects_per10' in d: d['beyond_something'] = int(d['beyond_objects_per10'] >= 4 or d.get('beyond_ground_share', 0) >= 0.5)
    return d


RULES = [
    # ---- shape and size
    ('A1 symmetric outline', 'the outline mirrors about its middle line', [('sym_best', 0.90, None, '1.00 (median)')]),
    ('A2 compact outline', 'no arms, no corridors: the outline fills its convex hull, and is not long', [('convexity', 0.85, None, '0.93 to 1.00'), ('aspect', None, 1.45, '1.00 to 1.20')]),
    ('A3 size', 'short side and floor area inside the real band', [('short_side', 'p10', 'p90', 'p25..p75'), ('area', 'p10', 'p90', 'p25..p75')]),
    ('A4 room for the boss', 'the floor is several bosses wide', [('span_per_boss', 'p10', None, 'p25..p75'), ('tiles_per_boss_tile', 'p10', 'p90', 'p25..p75')]),
    ('A5 clear fighting floor', 'a clear square of floor at least 1.4 bosses a side (median 2.8)', [('clear_square_per_boss', 'p10', None, 'p25..p75')]),
    ('A6 boss on the middle line', 'the boss stands on the outline\'s middle line (centre, or at / beyond the far edge on the axis)', [('boss_axis_off', None, 1.5, '0')]),
    ('A7 entry away from the boss', 'players come in at the edge, at least 3 tiles from the boss', [('entry_to_boss', 3, None, 'p25..p75')]),
    # ---- floor
    ('F1 few floor kinds', 'at most four kinds of ground make 95% of the floor', [('floor_kinds_95', None, 4, '1 to 4')]),
    ('F2 flat', 'at least four tiles in five are dead flat', [('flat_share', 0.80, None, '1.00')]),
    ('F3 one field', 'the floor reads as one field: most of it one colour, few colours', [('pic_floor_plain_share', 'p10', None, 'p25..p75'), ('pic_floor_colours_90', None, 'p90', 'p25..p75')]),
    ('F4 marking is symmetric', 'where the floor is marked (5% or more of it), the marking mirrors', [('mark_sym', 0.60, None, '0.96 (median)')]),
    ('F5 floor not vivid', 'floor saturation inside the real band', [('pic_floor_sat', None, 'p90', 'p25..p75')]),
    # ---- edge
    ('E1 an edge you can see', 'the outline is bounded by something seen: built (walls, rock, pillars) on the edge or one to two tiles behind it, a drop into dark, or other ground (water, lava)', [('edge_seen', 0.80, None, '0.95 to 1.00')]),
    ('E2 edge dressed', 'objects within two tiles of the outline, per 10 outline edges', [('edge_objects_per10', 'p10', None, 'p25..p75')]),
    ('E3 few kinds, repeated', 'a few kinds of edge piece make 80% of the edge, each used several times', [('edge_kinds_80', None, 'p90', 'p25..p75'), ('edge_repeat', 'p10', None, 'p25..p75')]),
    ('E4 tall pieces at the edge', 'standing pieces 200 units or taller in the edge, per 10 outline edges; the edge\'s tall things over a tile high', [('edge_tall_per10', 'p10', None, 'p25..p75'), ('edge_height_p90', 130, None, 'p25..p75')]),
    ('E5 little hung on walls', 'wall decoration objects per 10 outline edges (the game models wall detail into the wall pieces)', [('edge_decor_per10', None, 0.5, '0')]),
    ('E6 a world beyond the edge', 'three to six tiles outside the outline there is ground, or at least 4 objects per 10 outline edges (rock, building, forest, the tower below)', [('beyond_something', 1, None, '1'), ('beyond_objects_per10', None, None, 'p25..p75')]),
    # ---- inside
    ('I1 open floor', 'the inside of the outline is walkable; objects block almost none of it', [('open_share', 0.90, None, '0.99 to 1.00'), ('blocked_by_objects', None, 0.05, '0')]),
    ('I2 few things inside', 'standing objects inside the outline per 100 tiles', [('inside_per100', None, 'p90', '0 to 1.4')]),
    ('I3 centre empty', 'nothing stands in the centre zone', [('inside_centre', None, 0, '0')]),
    # ---- palette and focus
    ('P1 boss off the floor\'s colour', 'colour distance (delta E) between the boss and the floor under him', [('pic_deltaE_mean', 15, None, 'p25..p75')]),
    ('P2 boss off the floor\'s brightness', 'brightness difference boss / floor under him (0..255)', [('pic_abs_luma_contrast', 13, None, 'p25..p75')]),
    ('P3 edge not brighter than the floor', 'surround brightness minus floor brightness', [('pic_surround_minus_floor_luma', None, 'p90', 'p25..p75')]),
    ('P4 vivid colour at the edge', 'share of the most vivid 2% of the picture that lies outside the floor', [('pic_accent_surround', 0.50, None, '0.53 to 1.00')]),
]


def limit(v, key, B):
    if v is None or isinstance(v, (int, float)): return v
    return B.get(key, {}).get(v)


def evaluate(d, B):
    """[(rule id, state, text, [(measure, value, low, high, real p25, median, p75, ok)])]; state: 'holds' | 'OUTSIDE' | 'n/a'"""
    out = []
    for rid, text, tests in RULES:
        rows = []; state = 'n/a'
        for key, lo, hi, aim in tests:
            v = d.get(key); b = B.get(key, {})
            lo_, hi_ = limit(lo, key, B), limit(hi, key, B)
            if v is None: rows.append((key, None, lo_, hi_, b.get('p25'), b.get('med'), b.get('p75'), None)); continue
            ok = (lo_ is None or v >= lo_ - 1e-9) and (hi_ is None or v <= hi_ + 1e-9)
            rows.append((key, v, lo_, hi_, b.get('p25'), b.get('med'), b.get('p75'), ok))
            state = 'OUTSIDE' if (not ok or state == 'OUTSIDE') else 'holds'
        out.append((rid, state, text, rows))
    return out


def load_bands():
    return json.loads((HERE / 'data' / 'bands.json').read_text())


def counts(D, B):
    res = {}
    for k, m in D.items():
        for rid, state, text, rows in evaluate(flat(m), B):
            r = res.setdefault(rid, dict(holds=[], outside=[], na=[], text=text))
            r['holds' if state == 'holds' else 'outside' if state == 'OUTSIDE' else 'na'].append(k)
    return res


if __name__ == '__main__':
    D = json.loads((HERE / 'data' / 'arenas.json').read_text()); bb = load_bands(); B = bb['bands']
    f = lambda v: '-' if v is None else (str(int(v)) if float(v).is_integer() else f'{v:.2f}')
    for rid, r in counts(D, B).items():
        n = len(r['holds']) + len(r['outside'])
        tests = next(t for i, _, t in RULES if i == rid)
        lim = '; '.join(f"{k} {f(limit(lo, k, B))}..{f(limit(hi, k, B))} (real {f(B[k]['p25'])} / {f(B[k]['med'])} / {f(B[k]['p75'])})" for k, lo, hi, aim in tests if k in B)
        print(f"{rid}: holds for {len(r['holds'])} of {n}" + (f" ({len(r['na'])} not measurable)" if r['na'] else '') + f"\n    {lim}\n    not: {', '.join(r['outside']) or '-'}")
