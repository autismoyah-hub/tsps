"""table.py: data/arenas.json -> data/bands.json (the real band of every measure) and data/tables.md (every arena, every table).

  table.py            writes both and prints the bands
Bands: p10, p25, median, p75, p90, min, max, n over the measured arenas. Picture measures are flattened to pic_*.
"""
import sys, json, math, pathlib, collections
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
from arenamodel import pct
from rules import flat


def flat_old(m):
    d = {k: v for k, v in m.items() if isinstance(v, (int, float)) and not isinstance(v, bool)}
    p = m.get('pic') or {}
    for k, v in p.items():
        if isinstance(v, (int, float)): d['pic_' + k] = v
        elif isinstance(v, dict):
            for kk, vv in v.items():
                if isinstance(vv, (int, float)): d[f'pic_{k}_{kk}'] = vv
    if p.get('floor') and p.get('surround'): d['pic_surround_minus_floor_luma'] = p['surround']['luma'] - p['floor']['luma']
    if p.get('floor_centre_luma') is not None and p.get('floor_band_luma') is not None: d['pic_centre_minus_band_luma'] = p['floor_centre_luma'] - p['floor_band_luma']
    if p.get('boss') and p.get('floor'): d['pic_boss_minus_floor_luma'] = p['boss']['luma'] - p['floor']['luma']
    r = (m.get('room') or {}).get('all', {}).get('measures') or {}
    for k, v in r.items():
        if isinstance(v, (int, float)): d['room_' + k] = v
    return d


def bands(rows):
    keys = sorted({k for r in rows for k in r})
    B = {}
    for k in keys:
        v = [r[k] for r in rows if r.get(k) is not None]
        if len(v) < 3: continue
        B[k] = dict(p10=pct(v, .1), p25=pct(v, .25), med=pct(v, .5), p75=pct(v, .75), p90=pct(v, .9), min=min(v), max=max(v), n=len(v))
    return B


def f(v, nd=2):
    if v is None: return '-'
    if isinstance(v, str): return v
    if isinstance(v, int) or float(v).is_integer(): return str(int(v))
    return f'{v:.{nd}f}'


def table(D, cols, title, heads=None):
    out = [f'### {title}', '', '| arena | ' + ' | '.join(heads or cols) + ' |', '|---|' + '---|' * len(cols)]
    for k, m in D.items():
        fl = flat(m); row = []
        for c in cols:
            v = fl.get(c, m.get(c))
            row.append(f(v) if not isinstance(v, (list, dict)) else str(v))
        out.append(f'| {k} | ' + ' | '.join(row) + ' |')
    return '\n'.join(out) + '\n'


def main():
    D = json.loads((HERE / 'data' / 'arenas.json').read_text())
    rows = [flat(m) for m in D.values()]
    B = bands(rows)
    (HERE / 'data' / 'bands.json').write_text(json.dumps(dict(n=len(rows), bands=B), indent=1))
    T = []
    T.append(table(D, ['name', 'bbox', 'plane', 'src'], 'Where'))
    T.append(table(D, ['area', 'hull', 'w', 'h', 'outline', 'box_fill', 'disc_iou', 'convexity', 'sym_best', 'inradius', 'clear_square', 'open_share'], 'Shape and size'))
    T.append(table(D, ['boss_side', 'tiles_per_boss_tile', 'span_per_boss', 'clear_square_per_boss', 'boss_place', 'boss_off_centre', 'boss_to_edge', 'n_entries', 'entry_to_boss'], 'The boss in it'))
    T.append(table(D, ['n_underlays', 'n_overlays', 'floor_kinds', 'floor_kinds_95', 'plain_share', 'marked_share', 'centre_marked', 'border_marked', 'ring_changes', 'concentric_gain', 'mark_sym', 'flat_share', 'height_range', 'height_total', 'floor_decor_per100', 'floor_decor_kinds', 'floor_decor_cover'], 'Floor'))
    T.append(table(D, ['edges', 'edge_main', 'edge_wall', 'edge_object', 'edge_void', 'edge_drop', 'edge_rise', 'edge_other_ground', 'edge_set_back', 'edge_unseen', 'edge_clip', 'edge_seen', 'edge_per_sqrt_area'], 'Edge: what bounds it (share of outline edges)'))
    T.append(table(D, ['edge_objects', 'edge_objects_per10', 'edge_kinds', 'edge_kinds_80', 'edge_top_share', 'edge_repeat', 'edge_height_med', 'edge_height_p90', 'edge_height_max', 'edge_tall_standing', 'edge_tall_per10', 'edge_decor_per10', 'edge_animated', 'edge_lights'], 'Edge: how it is dressed'))
    T.append(table(D, ['inside_objects', 'inside_blockers', 'inside_per100', 'blocked_by_objects', 'blocked_other', 'inside_centre', 'inside_middle', 'inside_band', 'inside_kinds', 'inside_max_repeat', 'inside_sym', 'inside_tall', 'inside_animated', 'inside_lights'], 'Objects inside the outline'))
    T.append(table(D, ['pic_floor_luma', 'pic_floor_luma_sd', 'pic_floor_sat', 'pic_floor_colours_90', 'pic_boss_luma', 'pic_boss_sat', 'pic_luma_contrast', 'pic_deltaE_mean', 'pic_boss_far_share', 'pic_surround_minus_floor_luma', 'pic_centre_minus_band_luma', 'pic_accent_centre', 'pic_accent_middle', 'pic_accent_band', 'pic_accent_surround'], 'Palette and focus (from the straight-down pictures)'))
    hue = ['### Colours by name', '', '| arena | floor | under the boss | boss | surround | floor kinds (share, colour) |', '|---|---|---|---|---|---|']
    for k, m in D.items():
        p = m.get('pic') or {}
        g = lambda s: (f"{p[s]['hue']} #{'%02x%02x%02x' % tuple(p[s]['rgb'])}" if p.get(s) else '-')
        fk = '; '.join(f"{c['kind']} {c['share']:.2f} {c['rgb']}" + (f" tex {c['texture']}" if c['texture'] is not None else '') for c in m['floor_colours'][:3])
        hue.append(f"| {k} | {g('floor')} | {g('under_boss')} | {g('boss')} | {g('surround')} | {fk} |")
    T.append('\n'.join(hue) + '\n')
    names = ['### What stands at the edge and inside (named pieces and game symbols)', '', '| arena | entries | inside | edge |', '|---|---|---|---|']
    for k, m in D.items():
        names.append(f"| {k} | {'; '.join(f'{e[0]} ({e[1]}) at ({e[2]},{e[3]})' for e in m['entries']) or '-'} | {', '.join(m['inside_named']) or '-'} | {', '.join(m['edge_named']) or '-'} |")
    T.append('\n'.join(names) + '\n')
    # room rules
    rr = collections.OrderedDict()
    for k, m in D.items():
        for tag in ('named', 'all'):
            for ok, rid in (m['room'].get(tag, {}).get('rules') or []):
                rr.setdefault(rid, {'named': [], 'all': []})[tag].append((k, ok))
    rt = ['### ROOMS.md rules run on the arenas (roomcheck.check, kind "any")', '',
          '"named": only what the game names counts as furniture, as in the rooms study. "all": every standing object counts.', '',
          '| room rule | arenas passing (named) | arenas passing (all) | missing it (all) |', '|---|---|---|---|']
    for rid, v in rr.items():
        pn = sum(ok for _, ok in v['named']); pa = sum(ok for _, ok in v['all'])
        rt.append(f"| {rid} | {pn} of {len(v['named'])} | {pa} of {len(v['all'])} | {', '.join(k for k, ok in v['all'] if not ok)} |")
    T.append('\n'.join(rt) + '\n')
    bt = ['### Bands', '', '| measure | p10 | p25 | median | p75 | p90 | min | max | n |', '|---|---|---|---|---|---|---|---|---|']
    for k, b in B.items():
        bt.append(f"| {k} | {f(b['p10'])} | {f(b['p25'])} | {f(b['med'])} | {f(b['p75'])} | {f(b['p90'])} | {f(b['min'])} | {f(b['max'])} | {b['n']} |")
    T.append('\n'.join(bt) + '\n')
    (HERE / 'data' / 'tables.md').write_text('\n'.join(T), encoding='utf-8')
    print('\n'.join(bt))
    json.dump({rid: dict(named=[sum(ok for _, ok in v['named']), len(v['named'])], all=[sum(ok for _, ok in v['all']), len(v['all'])]) for rid, v in rr.items()},
              open(HERE / 'data' / 'room-rules.json', 'w'), indent=1)


if __name__ == '__main__':
    main()
