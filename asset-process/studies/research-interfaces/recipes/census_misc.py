"""census_misc.py: the conventions that are not frame families, counted over every drawn 239 group (held-out
excluded): wells and backings outside frames, text emphasis, line pitch, button and tab labels, bars, scrollbars,
rule lines, two-tone outlines, close buttons. Writes recipes/out/misc.json and prints a summary."""
import sys, json, re, collections
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
import common as K
import census_frames as F
import ifcache as C

cache = C.Cache(C.CACHE_242)
FN = {494: 'p11', 495: 'p12', 496: 'b12', 497: 'q8'}
BACKS = {297: 'TRADEBACKING', 1040: 'LIGHT', 897: 'DARK', 1545: 'BEIGE'}
CNT = collections.Counter


def c6(v):
    return f'{v & 0xffffff:06x}'


def inside(a, b, tol=0):
    return a[0] >= b[0] - tol and a[1] >= b[1] - tol and a[0] + a[2] <= b[0] + b[2] + tol and a[1] + a[3] <= b[1] + b[3] + tol


def rect(r):
    return r['abs'] + r['size']


M = dict(wells=collections.defaultdict(CNT), well_examples=collections.defaultdict(list), emph=CNT(), emph_groups=collections.defaultdict(set),
         emph_pos=CNT(), lh=CNT(), lh_groups=collections.defaultdict(set), btn=CNT(), btn_groups=collections.defaultdict(set),
         btnbox=CNT(), bars=CNT(), bar_groups=collections.defaultdict(set), scroll=CNT(), scroll_groups=set(), lines=CNT(),
         twotone=CNT(), twotone_groups=collections.defaultdict(set), heading=CNT(), heading_groups=collections.defaultdict(set),
         textbox=CNT(), close=CNT(), close_groups=collections.defaultdict(set), multiline_align=CNT())
fam_names = list(F.FAMILIES)
for g, rows, by in K.all_states():
    vis = [r for r in rows if r['visible']]
    kids = collections.defaultdict(list)
    for r in vis:
        kids[r['parent']].append(r)
    insts = []
    member = {}
    for name in fam_names:
        for inst in F.find_instances(rows, by, F.FAMILIES[name]):
            inst['family'] = name
            insts.append(inst)
            for m in inst['members']:
                member[m['id']] = inst
    # ---- wells and backings outside frame instances
    for r in vis:
        if r['kind'] != 5 or r['sprites'][0] not in BACKS or not r['sprite_tiling'] or r['id'] in member:
            continue
        s = BACKS[r['sprites'][0]]
        p = by.get(r['parent'])
        rb = rect(r)
        place = 'other'
        if p:
            pb = rect(p)
            d = [rb[0] - pb[0], rb[1] - pb[1], pb[0] + pb[2] - rb[0] - rb[2], pb[1] + pb[3] - rb[1] - rb[3]]
            place = 'fills parent' if d == [0, 0, 0, 0] else f'parent inset {d[0]}' if len(set(d)) == 1 else 'part of parent'
        box = next((i for i in insts if inside(rb, i['box']) and i['parent'] == r['parent']), None)
        boxrel = None
        if box:
            bx = box['box']
            boxrel = (box['family'], rb[0] - bx[0], rb[1] - bx[1], 'W' + str(rb[2] - bx[2]), rb[3])
            # drawn before or after the box's pieces
            first = min(m['order'] for m in box['members'])
            last = max(m['order'] for m in box['members'])
            when = 'before frame' if r['order'] < first else 'after frame' if r['order'] > last else 'between'
        else:
            when = '-'
        key = (s, r['transparency'], int(r['flip_h']), int(r['flip_v']), place, when)
        M['wells'][key][g] += 1
        if boxrel and len(M['well_examples'][(s, boxrel[0])]) < 30:
            M['well_examples'][(s, boxrel[0])].append((g, boxrel, r['transparency'], int(r['flip_h']), when))
    # ---- texts
    for r in vis:
        if r['kind'] != 4 or r['font'] < 0 or not r['text']:
            continue
        t = r['text']
        f = FN.get(r['font'], r['font'])
        base = c6(r['colors'][0])
        for m in re.finditer(r'<col=([0-9a-fA-F]{6})>(.*?)(</col>|<col=|$)', t):
            M['emph'][(f, base, m.group(1).lower())] += 1
            M['emph_groups'][(f, base, m.group(1).lower())].add(g)
            pos = 'whole text' if m.start() == 0 and m.end() >= len(t) - 6 else 'at start' if m.start() == 0 else 'inside'
            M['emph_pos'][(base, m.group(1).lower(), pos)] += 1
        nlines = t.count('<br>') + 1
        if nlines > 1:
            M['lh'][(f, r['line_height'])] += 1
            M['lh_groups'][(f, r['line_height'])].add(g)
            M['multiline_align'][(f, tuple(r['text_alignment']))] += 1
        M['textbox'][(f, r['size'][1], tuple(r['text_alignment']))] += 1
        # headings: b12 texts that are not frame titles
        if r['font'] == 496:
            p = by.get(r['parent'])
            is_title = any(i['family'] in ('steel', 'stone', 'stone2') and r['abs'][0] - i['box'][0] == 6 and
                           r['abs'][1] - i['box'][1] in (6, 7) for i in insts)
            if not is_title:
                M['heading'][(base, int(r['text_shadow']), tuple(r['text_alignment']), r['size'][1])] += 1
                M['heading_groups'][(base, int(r['text_shadow']))].add(g)
    # ---- button / tab labels: texts whose box lies inside a small framed box (<= 200x60) of a family
    for inst in insts:
        bx = inst['box']
        if inst['family'] in ('steel', 'stone', 'stone2', 'thin') or bx[2] > 200 or bx[3] > 60:
            continue
        for r in vis:
            if r['kind'] != 4 or r['font'] < 0 or not r['text']:
                continue
            tb = rect(r)
            if inside(tb, bx, 1):
                key = (inst['family'], FN.get(r['font'], r['font']), c6(r['colors'][0]), int(r['text_shadow']),
                       tuple(r['text_alignment']))
                M['btn'][key] += 1
                M['btn_groups'][key].add(g)
                M['btnbox'][(inst['family'], 'same box' if tb == bx else f'inset {tb[0] - bx[0]},{tb[1] - bx[1]},'
                             f'{bx[2] - tb[2]},{bx[3] - tb[3]}')] += 1
    # ---- bars: stacks of opaque filled rectangles <= 21 tall sharing top and height, in draw order
    fills = [r for r in vis if r['kind'] == 3 and r['filled'] and r['transparency'] < 255 and 3 <= r['size'][1] <= 21]
    used = set()
    for r in fills:
        if r['id'] in used:
            continue
        same = [q for q in fills if q['abs'][1] == r['abs'][1] and q['size'][1] == r['size'][1] and q['abs'][0] == r['abs'][0]]
        if len(same) < 2:
            continue
        for q in same:
            used.add(q['id'])
        same.sort(key=lambda q: q['order'])
        seq = tuple((c6(q['colors'][0]), 'full' if q['size'][0] == max(x['size'][0] for x in same) else 'part',
                     q['transparency']) for q in same)
        rim = [o for o in vis if o['kind'] == 3 and not o['filled'] and o['abs'][0] <= r['abs'][0] - 1
               and o['abs'][1] <= r['abs'][1] - 1 and o['abs'][0] + o['size'][0] >= r['abs'][0] + max(x['size'][0] for x in same) + 1
               and o['size'][1] <= r['size'][1] + 6]
        key = (seq, c6(rim[0]['colors'][0]) if rim else None)
        M['bars'][key] += 1
        M['bar_groups'][key].add(g)
    # ---- scrollbars
    for r in vis:
        if r['kind'] == 5 and r['sprites'][0] == 792:
            p = by.get(r['parent'])
            if not p:
                continue
            pieces = sorted([(q['sprites'][0], q['abs'][0] - p['abs'][0], q['abs'][1] - p['abs'][1], q['size'][0], q['size'][1])
                             for q in kids[p['id']] if q['kind'] == 5], key=lambda x: x[0])
            H = p['size'][1]
            sig = tuple((s, x, y if y < H / 2 else f'H{y - H:+d}', w, h if h < H / 2 else f'H{h - H:+d}')
                        for s, x, y, w, h in pieces if s in (773, 788, 792))
            M['scroll'][(p['size'][0], sig)] += 1
            M['scroll_groups'].add(g)
    # ---- lines
    for r in vis:
        if r['kind'] == 9:
            M['lines'][(c6(r['colors'][0]), r['line_width'], 'horizontal' if r['size'][1] == 0 else 'vertical' if r['size'][0] == 0 else 'diagonal')] += 1
    # ---- two-tone outlines: an unfilled rectangle with another exactly 1px inside
    outl = [r for r in vis if r['kind'] == 3 and not r['filled']]
    for a in outl:
        b = next((q for q in outl if q['abs'] == [a['abs'][0] + 1, a['abs'][1] + 1] and q['size'] == [a['size'][0] - 2, a['size'][1] - 2]), None)
        if b:
            key = (c6(a['colors'][0]), c6(b['colors'][0]), a['transparency'], b['transparency'])
            M['twotone'][key] += 1
            M['twotone_groups'][key].add(g)
    # ---- close buttons
    for r in vis:
        if r['kind'] == 5 and r['sprites'][0] in (535, 539, 541, 537, 831, 2152):
            host = next((i for i in insts if i['id'] == r['parent']), None) if False else None
            fr = next((i for i in insts if inside(rect(r), i['box']) and i['family'] in ('steel', 'stone', 'thin')), None)
            rel = (r['abs'][0] - fr['box'][0] - fr['box'][2], r['abs'][1] - fr['box'][1]) if fr else None
            key = (r['sprites'][0], tuple(r['size']), fr['family'] if fr else None, rel)
            M['close'][key] += 1
            M['close_groups'][key].add(g)


def top(c, n=25, groups=None):
    out = []
    for k, v in c.most_common(n):
        e = [k, v]
        if groups is not None:
            e.append(len(groups.get(k, ())))
        out.append(e)
    return out


R = {}
W = []
for k, gs in M['wells'].items():
    W.append([list(k), sum(gs.values()), len(gs), sorted(gs)[:10]])
W.sort(key=lambda e: -e[2])
R['wells'] = W
R['well_examples'] = {str(k): v for k, v in M['well_examples'].items()}
R['emphasis'] = top(M['emph'], 30, M['emph_groups'])
R['emphasis_position'] = top(M['emph_pos'], 20)
R['line_height_multiline'] = top(M['lh'], 25, M['lh_groups'])
R['multiline_align'] = top(M['multiline_align'], 15)
R['button_labels'] = top(M['btn'], 30, M['btn_groups'])
R['button_label_box'] = top(M['btnbox'], 20)
R['bars'] = top(M['bars'], 25, M['bar_groups'])
R['scrollbars'] = top(M['scroll'], 15)
R['scrollbar_groups'] = len(M['scroll_groups'])
R['lines'] = top(M['lines'], 15)
R['twotone'] = top(M['twotone'], 15, M['twotone_groups'])
R['headings_b12_not_titles'] = top(M['heading'], 20)
R['heading_groups'] = top(CNT({k: len(v) for k, v in M['heading_groups'].items()}), 10)
R['text_boxes'] = top(M['textbox'], 30)
R['close_buttons'] = top(M['close'], 20, M['close_groups'])
(K.HERE / 'recipes' / 'out' / 'misc.json').write_text(json.dumps(R, indent=1, default=str))
for k, v in R.items():
    print('##', k)
    if isinstance(v, list):
        for e in v[:22]:
            print('  ', e)
    elif isinstance(v, dict):
        for kk, vv in list(v.items())[:8]:
            print('  ', kk, vv[:6])
    else:
        print('  ', v)
