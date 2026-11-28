"""ifgate.py: the gate for interfaces (NEXT.md section 1), and the convention checks for ORIGINAL interfaces.

  ifgate.py rebuild GROUP REBUILD.json [--detail] [--out DIR]
      A blind rebuild of real panel GROUP. Three parts:
      (a) PIXELS: iface.py draws the rebuild and the real panel (the state TSPS showed; cache 242; TSPS blend) in
          the same pane, each on two backgrounds; a pixel differs if it differs on either. Share = differing pixels
          anywhere in the pane, outside the real panel's model and item rectangles, over the real panel's area.
          PASS <= 0.5% or 100 px, BORDERLINE <= 2% or 300 px (the pixel floors matter only for panels under
          20,000 px, e.g. the HUD). Board: real | rebuild | difference (magenta).
      (b) STRUCTURE (v2: a component that changes no pixel, e.g. a rectangle entirely covered by a later opaque
          one, is not scored on either side): every drawn text of the real panel has a rebuild text with the same words, font, colour (glyph
          by glyph, <col> markup included) and shadow whose letters' box is within 2px; every border, divider and
          close-button sprite of the real panel is drawn by the same sprite within 2px in the rebuild (and the
          rebuild draws none the real one does not); every visible rectangle and line has one of the same kind,
          colour, fill and transparency within 2px; every tiled backing or well has one of the same sprite,
          flips and transparency within 2px. Icons (v2): BORDERLINE at worst when not matched (a rules sheet
          cannot name them); models: INFO.
      (c) CONSTRUCTION: one root and every component under it; only kinds 0,3,4,5,6,9; only sprites and fonts that
          exist in the cache (no server sprites); fits the 512x334 modal; nothing drawn outside the root; converts
          with to_tsps.py.
      (d) PER-REGION VERDICTS (v2, INFO): one line per sheet region R1.. and string T1..: matched / not matched
          (every real component in it matched by (b), and no differing pixel where it is on top); no further detail.
      Without --detail (the RULES-sheet run) it prints counts only and writes the board to the Teacher's folder;
      with --detail (picture run, Teacher) it names every unmatched item and writes the board next to REBUILD.json.

  ifgate.py original ORIGINAL.json [--out DIR]
      An ORIGINAL panel or HUD with no real twin ("overlay": true in the JSON for a HUD). Construction checks plus
      the measured conventions of INTERFACES.md, each with the real band; INFO where the real panels have
      exceptions.

  ifgate.py draw COMPONENTS.json OUT.png [--pane modal|hud] [--scale 3]
      Draw a component list exactly as the gate does (cache 242, TSPS blend) in its pane: OUT.png is the pane,
      OUT-crop3x.png the covered part at 3x. Reads no target data; authors may run it at will.

Every line: measure | value | threshold | verdict. Last line: VERDICT PASS|BORDERLINE|FAIL (worst scored line).
"""
import sys, json, argparse, collections
from pathlib import Path
import numpy as np
from PIL import Image

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import ifcache as C
import iface
import ifref as R
import to_tsps

LEVELS = ['PASS', 'BORDERLINE', 'FAIL', 'INFO']
PIX_PASS, PIX_EDGE = 0.5, 2.0
# Floors in pixels (proposed from the damage tests, ifgate_proof.py): on a small HUD (303: 5,920 px) one text 1px
# lower, which structure accepts, is already 2.9%. A share alone punishes small panels; the floors stop that.
PIX_PASS_FLOOR, PIX_EDGE_FLOOR = 100, 300
TOL = 2


class Lines:
    def __init__(self, quiet=False):
        self.rows, self.worst, self.quiet = [], 0, quiet

    def add(self, name, value, threshold, verdict):
        self.rows.append(dict(measure=name, value=value, threshold=threshold, verdict=verdict))
        if verdict in LEVELS[:3]:
            self.worst = max(self.worst, LEVELS.index(verdict))
        if not self.quiet:
            v = value if isinstance(value, str) else json.dumps(value)
            print(f'{name} | {v} | {threshold} | {verdict}', flush=True)

    def le(self, name, value, good, edge, threshold=None, show=None):
        self.add(name, show if show is not None else value, threshold or f'PASS <= {good:g}, BORDERLINE <= {edge:g}',
                 'PASS' if value <= good else 'BORDERLINE' if value <= edge else 'FAIL')

    def all_of(self, name, got, total, extra='', fail='FAIL'):
        v = f'{got} of {total}' + (f'; {extra}' if extra else '')
        self.add(name, v, 'PASS = all', 'PASS' if got == total else fail)

    def info(self, name, value, band):
        self.add(name, value, band, 'INFO')

    def verdict(self):
        if not self.quiet:
            print('VERDICT', LEVELS[self.worst], flush=True)
        return LEVELS[self.worst]


def dilate(m, r=TOL):
    h, w = m.shape
    p = np.pad(m, r)
    out = np.zeros_like(m)
    for dy in range(2 * r + 1):
        for dx in range(2 * r + 1):
            out |= p[dy:dy + h, dx:dx + w]
    return out


def near(a, b, tol=TOL):
    return a is not None and b is not None and all(abs(a[k] - b[k]) <= tol for k in (0, 1)) and \
        abs((a[0] + a[2]) - (b[0] + b[2])) <= tol and abs((a[1] + a[3]) - (b[1] + b[3])) <= tol


# ---------------------------------------------------------------- construction (shared by rebuild and original)

def construction(L, comps, S, pane_kind, detail):
    files = [c['file'] for c in comps]
    ids = {c['id'] for c in comps}
    roots = [c for c in comps if c['parent'] is None]
    bad_parent = [c['file'] for c in comps if c['parent'] is not None and c['parent'] not in ids]
    reach = set(S.lay)
    L.add('one root, every component under it', f'{len(roots)} root(s), {len(bad_parent)} with a missing parent, '
          f'{len(comps) - len(reach)} not reached', 'PASS = 1, 0, 0',
          'PASS' if len(roots) == 1 and not bad_parent and len(reach) == len(comps) else 'FAIL')
    dup = [f for f, n in collections.Counter(files).items() if n > 1]
    kinds = collections.Counter(c['kind'] for c in comps)
    badk = {k: n for k, n in kinds.items() if k not in (0, 3, 4, 5, 6, 9)}
    L.add('file numbers unique, kinds 0/3/4/5/6/9 only', f'{len(dup)} duplicate(s), other kinds {badk or "none"}',
          'PASS = none', 'PASS' if not dup and not badk else 'FAIL')
    cache = R.cache()
    bad_s, bad_f = set(), set()
    for c in comps:
        if c['kind'] == 5:
            s = c['sprites'][0]
            if s == -1:   # an empty graphic (draws nothing; real panels have them, e.g. item slots)
                continue
            if s < 0 or s >= 30000:
                bad_s.add(s); continue
            try:
                C.sprite_sheet(cache, s)
            except Exception:
                bad_s.add(s)
        if c['kind'] == 4 and not (c['font'] == -1 and not c.get('text')):   # an empty text slot draws nothing
            try:
                C.font(cache, c['font'])
            except Exception:
                bad_f.add(c['font'])
    L.add('sprites and fonts from the cache', f'{len(bad_s)} bad sprite(s) {sorted(bad_s)[:6]}, {len(bad_f)} bad '
          f'font(s) {sorted(bad_f)[:6]}', 'PASS = none (cache 242; no server sprites 30000+)',
          'PASS' if not bad_s and not bad_f else 'FAIL')
    cov = S.covered()
    pr = S.panel_rect()
    root = roots[0] if roots else None
    rsz = S.lay[root['id']]['size'] if root else [0, 0]
    rab = S.lay[root['id']]['abs'] if root else [0, 0]
    if pane_kind == 'modal':
        ok = rsz[0] <= 512 and rsz[1] <= 334 and pr[2] <= 512 and pr[3] <= 334
        L.add('fits the 512x334 modal', f'root {rsz[0]}x{rsz[1]}, drawn {pr[2]}x{pr[3]}', 'PASS <= 512x334',
              'PASS' if ok else 'FAIL')
    else:
        ok = pr[2] <= 512 and pr[3] <= 334
        L.add('fits the HUD pane (drawn part at most 512x334)', f'drawn {pr[2]}x{pr[3]} at {pr[:2]}',
              'PASS <= 512x334', 'PASS' if ok else 'FAIL')
    out = cov.copy()
    out[max(rab[1], 0):max(rab[1] + rsz[1], 0), max(rab[0], 0):max(rab[0] + rsz[0], 0)] = False
    dead = 0
    for c in comps:
        p = S.lay.get(c['id'])
        if p and c is not root and c['kind'] != 0 and (p['clip'][0] >= p['clip'][2] or p['clip'][1] >= p['clip'][3]):
            dead += 1
    L.add('nothing drawn outside the root', f'{int(out.sum())} pixels outside; {dead} component(s) clipped away',
          'PASS = 0 pixels', 'PASS' if not out.any() else 'FAIL')
    try:
        to_tsps.convert(comps, comps[0]['group'])
        conv = 'converts'
    except Exception as e:
        conv = f'fails: {e!r}'[:120]
    L.add('to_tsps.py conversion', conv, 'PASS = converts', 'PASS' if conv == 'converts' else 'FAIL')
    nl = sum(1 for c in comps if c.get('listeners'))
    L.info('components with listeners (TSPS custom interfaces drop them)', nl, 'INFO')


# ---------------------------------------------------------------- rebuild

def texts_of(S):
    out = []
    for i in S.drawable():
        w = S.byid[i]
        if w['kind'] != 4 or w['font'] < 0:
            continue
        box = S.ink(i)[1]
        if box is None:
            continue
        out.append(dict(id=i, text=R.plain(R.visible_text(w)), font=w['font'], shadow=bool(w['text_shadow']),
                        runs=[c for _, c in R.colour_runs(w)], ink=box))
    return out


def match_texts(SR, SB):
    real, reb = texts_of(SR), texts_of(SB)
    free = list(reb)
    missing = []
    for t in real:
        best = None
        for u in free:
            if u['text'] != t['text']:
                continue
            fails = [k for k, ok in (('font', u['font'] == t['font']), ('colour', u['runs'] == t['runs']),
                                     ('shadow', u['shadow'] == t['shadow']), ('position', near(u['ink'], t['ink'])))
                     if not ok]
            if best is None or len(fails) < len(best[1]):
                best = (u, fails)
        if best and not best[1]:
            free.remove(best[0])
        else:
            missing.append((t, best[1] if best else ['missing (no text with these words)'],
                            best[0]['ink'] if best else None))
    return real, missing, free


def sprite_ink_check(SR, SB, ids_r, ids_b):
    """Real components ids_r (graphics) matched by rebuild graphics of the same sprite within TOL px; and rebuild
    graphics (of sprites in the same class) whose ink is not within TOL of the real ink of that sprite."""
    def union(S, ids):
        u = {}
        for i in ids:
            s = S.byid[i]['sprites'][0]
            m = S.ink(i)[0]
            if m is None:
                continue
            u[s] = u.get(s, np.zeros_like(m)) | m
        return u
    ur, ub = union(SR, ids_r), union(SB, ids_b)
    dr = {s: dilate(m) for s, m in ur.items()}
    db = {s: dilate(m) for s, m in ub.items()}
    missing, extra = [], []
    for i in ids_r:
        m = SR.ink(i)[0]
        if m is None:
            continue
        s = SR.byid[i]['sprites'][0]
        if s not in db or (m & ~db[s]).any():
            missing.append(i)
    for i in ids_b:
        m = SB.ink(i)[0]
        if m is None:
            continue
        s = SB.byid[i]['sprites'][0]
        if s not in dr or (m & ~dr[s]).any():
            extra.append(i)
    n = sum(1 for i in ids_r if SR.ink(i)[0] is not None)
    return n, missing, extra


def box_sprites(S):
    """Sprites of the boxes built of sprite pieces outside the frame families (e.g. 383's tab button): they are
    borders too."""
    regions, _ = R.classify(S)
    return {S.byid[m]['sprites'][0] for r in regions if r['family'] == 'sprite-built' for m in r['members']}


def graphics_by_class(S, extra=frozenset()):
    border, backing, other = [], [], []
    for i in S.drawable():
        w = S.byid[i]
        if w['kind'] != 5 or w.get('item', -1) != -1:   # item slots are excluded like models
            continue
        s = w['sprites'][0]
        if s in R.BORDERS or s in extra:
            border.append(i)
        elif s in R.BACKINGS and w['sprite_tiling']:
            backing.append(i)
        else:
            other.append(i)
    return border, backing, other


def match_simple(SR, SB, ids_r, ids_b, key):
    free = list(ids_b)
    missing = []
    for i in ids_r:
        k = key(SR, i)
        hit = next((j for j in free if key(SB, j) == k and near(SR.ink(i)[1], SB.ink(j)[1])), None)
        if hit is None:
            missing.append(i)
        else:
            free.remove(hit)
    return missing, free


def rect_key(S, i):
    w = S.byid[i]
    return (w['kind'], bool(w['filled']), w['colors'][0] & 0xffffff, w['transparency'], w.get('fill_mode', 0),
            w['line_width'] if w['kind'] == 9 else 0)


def sprite_key(S, i):
    w = S.byid[i]
    return (w['sprites'][0], bool(w['flip_h']), bool(w['flip_v']), w['transparency'])


def describe(S, i):
    w = S.byid[i]
    k = {3: 'rectangle', 4: 'text', 5: 'sprite', 6: 'model', 9: 'line'}.get(w['kind'], f'kind {w["kind"]}')
    b = S.ink(i)[1] if w['kind'] != 6 else S.lay[i]['abs'] + S.lay[i]['size']
    extra = f' {w["sprites"][0]}' if w['kind'] == 5 else ''
    return f'{k}{extra} at {b}'


def board(real, reb, diff, excl, rect, path, scale=2):
    x, y, w, h = rect
    a = real[y:y + h, x:x + w].astype(np.uint8)
    b = reb[y:y + h, x:x + w].astype(np.uint8)
    v = (reb[y:y + h, x:x + w] * 0.35).astype(np.uint8)
    d = diff[y:y + h, x:x + w]
    v[d] = [255, 0, 255]
    e = excl[y:y + h, x:x + w]
    v[e] = (v[e] * 0.5 + np.array([0, 60, 120])).astype(np.uint8)
    pad = 6
    out = np.full((h, 3 * w + 2 * pad, 3), 20, np.uint8)
    out[:, :w] = a; out[:, w + pad:2 * w + pad] = b; out[:, 2 * w + 2 * pad:] = v
    im = Image.fromarray(out)
    if scale > 1:
        im = im.resize((im.width * scale, im.height * scale), Image.NEAREST)
    im.save(path)


def gate_rebuild(group, comps, detail=False, out=None, quiet=False, board_path=None):
    L = Lines(quiet)
    SR = R.real_scene(group)
    comps = R.normalize(comps)
    SB = R.rebuild_scene(comps, SR.kind)
    ra, rb, _ = SR.images()
    ba, bb, _ = SB.images()
    excl, ex_rects = SR.excluded()
    diff = ((ra != ba).any(-1) | (rb != bb).any(-1)) & ~excl
    pr = SR.panel_rect()
    px, py, pw, ph = pr
    area = pw * ph - int(excl[py:py + ph, px:px + pw].sum())
    n = int(diff.sum())
    share = 100.0 * n / max(area, 1)
    good = max(PIX_PASS, 100.0 * PIX_PASS_FLOOR / max(area, 1))
    edge = max(PIX_EDGE, 100.0 * PIX_EDGE_FLOOR / max(area, 1))
    L.le('pixels differing, share of the panel (models and items left out)', share, good, edge,
         f'PASS <= {PIX_PASS}% or {PIX_PASS_FLOOR} px, BORDERLINE <= {PIX_EDGE}% or {PIX_EDGE_FLOOR} px'
         f' (here {good:.2f}%, {edge:.2f}%)', show=f'{share:.3f}% ({n} of {area})')
    bp = SB.panel_rect()
    L.info('panel rectangle in the pane, rebuild against real', f'{bp} against {pr}', 'INFO')
    # icons excluded: how much of the difference is the choice of icon pictures
    boxes = box_sprites(SR)
    _, _, other_r = graphics_by_class(SR, boxes)
    icon_m = np.zeros_like(excl)
    for i in other_r:
        b = SR.ink(i)[1]
        if b is None:   # an item slot drawn as a graphic: excluded already
            continue
        icon_m[b[1]:b[1] + b[3], b[0]:b[0] + b[2]] = True
    if icon_m.any():
        n2 = int((diff & ~icon_m).sum())
        L.info('pixels differing outside the real icons\' boxes', f'{100.0 * n2 / max(area, 1):.3f}% ({n2})', 'INFO')
    # board
    u = [min(pr[0], bp[0]), min(pr[1], bp[1]), max(pr[0] + pr[2], bp[0] + bp[2]), max(pr[1] + pr[3], bp[1] + bp[3])]
    u = [max(u[0] - 4, 0), max(u[1] - 4, 0), min(u[2] + 4, SR.pane[0]), min(u[3] + 4, SR.pane[1])]
    rect = [u[0], u[1], u[2] - u[0], u[3] - u[1]]
    if board_path:
        board(ra, ba, diff, excl, rect, board_path)
    # structure
    real_t, miss_t, extra_t = match_texts(SR, SB)
    why = collections.Counter(r for _, rs, _ in miss_t for r in rs)
    L.all_of('texts matched (words, font, colour, shadow, letters within 2px)', len(real_t) - len(miss_t), len(real_t),
             ', '.join(f'{k} {v}' for k, v in why.items()))
    br, bk_r, ot_r = graphics_by_class(SR, boxes)
    bb_, bk_b, ot_b = graphics_by_class(SB, boxes)
    nb, miss_b, extra_b = sprite_ink_check(SR, SB, br, bb_)
    L.all_of('border, divider, close and box-piece sprites present within 2px', nb - len(miss_b), nb,
             f'{len(extra_b)} drawn where the real panel has none')
    if extra_b and not miss_b:
        L.rows[-1]['verdict'] = 'FAIL'; L.worst = 2
        if not quiet:
            print('  (FAIL: extra border sprites)')
    rr = [i for i in SR.drawable() if SR.byid[i]['kind'] in (3, 9)]
    rbb = [i for i in SB.drawable() if SB.byid[i]['kind'] in (3, 9)]
    miss_r, extra_r = match_simple(SR, SB, rr, rbb, rect_key)
    L.all_of('rectangles and lines matched (kind, colour, fill, transparency, within 2px)', len(rr) - len(miss_r),
             len(rr), f'{len(extra_r)} extra in the rebuild')
    miss_o, _ = match_simple(SR, SB, ot_r, ot_b, sprite_key)
    # v2: scored as BORDERLINE at worst (not FAIL): a rules sheet names icons only by letter, so a rules run cannot
    # get them, but a swapped or wrong icon must not PASS (422's swapped icons did, at 0.39% of pixels)
    L.add('icons and other sprites: same sprite within 2px', f'{len(ot_r) - len(miss_o)} of {len(ot_r)}',
          'PASS = all, BORDERLINE otherwise (a rules sheet names icons only by letter)',
          'PASS' if not miss_o else 'BORDERLINE')
    miss_k, _ = match_simple(SR, SB, bk_r, bk_b, sprite_key)
    L.all_of('backings and wells matched (same sprite, flips, transparency, within 2px)', len(bk_r) - len(miss_k),
             len(bk_r))
    mr = [i for i in SR.order if SR.byid[i]['kind'] == 6 and not SR.lay[i]['hidden']]
    mb = [i for i in SB.order if SB.byid[i]['kind'] == 6 and not SB.lay[i]['hidden']]
    mm = sum(1 for i in mr if any(near(SR.lay[i]['abs'] + SR.lay[i]['size'], SB.lay[j]['abs'] + SB.lay[j]['size'])
                                  for j in mb))
    if mr or mb:
        L.info('model rectangles within 2px (models are not drawn)', f'{mm} of {len(mr)}; rebuild has {len(mb)}', 'INFO')
    # v2: per-region verdicts, numbered as on the sheet (the same classify procedure): a region or string is
    # matched when every real component in it is matched by the structure checks above and none of the pixels it
    # shows on top in the real drawing differs. No further detail (a rules run may see these lines).
    unmatched = {t['id'] for t, _, _ in miss_t} | set(miss_b) | set(miss_r) | set(miss_o) | set(miss_k)
    regions, rtexts = R.classify(SR)
    top = np.full(diff.shape, -1, np.int64)
    for i in SR.drawable():
        m = SR.ink(i)[0] if SR.byid[i]['kind'] != 6 else None
        if m is not None:
            top[m] = i
    verdicts = []
    for n, r in enumerate(regions, 1):
        if r['family'] in ('model', 'item', 'grid'):
            verdicts.append((f'R{n}', 'not scored'))
            continue
        bad = any(m in unmatched for m in r['members']) or bool((diff & np.isin(top, r['members'])).any())
        verdicts.append((f'R{n}', 'not matched' if bad else 'matched'))
    for n, t in enumerate(rtexts, 1):
        bad = t['id'] in unmatched or bool((diff & (top == t['id'])).any())
        verdicts.append((f'T{n}', 'not matched' if bad else 'matched'))
    for k, v in verdicts:
        L.info(f'region {k}', v, 'INFO (per-region verdict; numbering as on the sheet)')
    if detail and not quiet:
        for t, rs, ib in miss_t:
            print(f'  text "{t["text"][:50]}" letters {t["ink"]}: {", ".join(rs)}' + (f' (yours at {ib})' if ib else ''))
        for i in miss_b:
            print(f'  border missing: {describe(SR, i)}')
        for i in extra_b:
            print(f'  border extra in yours: {describe(SB, i)}')
        for i in miss_r:
            print(f'  rectangle/line missing: {describe(SR, i)}')
        for i in extra_r:
            print(f'  rectangle/line extra in yours: {describe(SB, i)}')
        for i in miss_o:
            print(f'  icon/sprite not matched: {describe(SR, i)}')
        for i in miss_k:
            print(f'  backing/well not matched: {describe(SR, i)}')
    construction(L, comps, SB, SR.kind, detail)
    v = L.verdict()
    return v, L.rows, dict(differ=n, area=area, share=share)


# ---------------------------------------------------------------- original

TEXT_COLOURS = {0xff981f: 2655, 0xffffff: 2016, 0xff0000: 1425, 0x9f9f9f: 423, 0x000000: 356, 0xffcc33: 150,
                0xebd5b2: 128, 0xffff00: 124, 0xbfbfbf: 121, 0xcfcfcf: 115, 0xff9933: 103, 0xccccff: 85,
                0xbb0000: 84, 0xf5e9d5: 82, 0xeaeaea: 0, 0x800000: 0, 0x0000ff: 0, 0x403020: 0, 0x46320a: 0}
DARK_TEXT = {0x000000, 0x403020, 0x46320a, 0x800000, 0x0000ff}
MAIN_FONTS = {494: 'p11_full', 495: 'p12_full', 496: 'b12_full', 497: 'q8_full'}
OTHER_FONTS = {645, 646, 647, 648, 1442, 1443, 1444, 1445, 1446, 1447, 764, 819, 5607, 5608, 5675, 5676, 6315}


def steel_227(W, H):
    """Script 227's pieces (INTERFACES.md 2.1): sprite -> rect relative to the frame's top-left."""
    return [(297, [1, 1, W - 2, H - 2]), (310, [0, 0, 25, 30]), (311, [W - 25, 0, 25, 30]), (312, [0, H - 30, 25, 30]),
            (313, [W - 25, H - 30, 25, 30]), (172, [-15, 30, 36, H - 60]), (315, [W - 21, 30, 36, H - 60]),
            (314, [25, -15, W - 50, 36]), (173, [25, H - 21, W - 50, 36]), (2546, [5, 14, W - 10, 26]),
            (535, [W - 29, 6, 26, 23])]


def gate_original(comps, overlay, quiet=False, grids=None):
    L = Lines(quiet)
    comps = R.normalize(comps)
    S = R.rebuild_scene(comps, 'hud' if overlay else 'modal')
    S.grids = grids or []
    construction(L, comps, S, 'hud' if overlay else 'modal', True)
    regions, texts = R.classify(S)
    draw = S.drawable()
    tw = [i for i in draw if S.byid[i]['kind'] == 4]
    # fonts and colours (both kinds)
    fonts = collections.Counter(S.byid[i]['font'] for i in tw)
    odd = {f: n for f, n in fonts.items() if f not in MAIN_FONTS}
    L.add('text fonts', ', '.join(f'{MAIN_FONTS.get(f, f)} x{n}' for f, n in fonts.items()) or 'no text',
          'PASS = p11/p12/b12/q8 only (95% of drawn text), BORDERLINE = other fonts in real use',
          'PASS' if not odd else 'BORDERLINE' if all(f in OTHER_FONTS for f in odd) else 'FAIL')
    cols = collections.Counter(c for i in tw for _, c in R.colour_runs(S.byid[i]))
    off = {f'{c:06x}': n for c, n in cols.items() if c not in TEXT_COLOURS}
    L.info('text colours outside the measured set', off or 'none',
           'INFO (set: ff981f 2655, ffffff 2016, ff0000 1425, 9f9f9f 423, 000000 356, ffcc33, ebd5b2, ffff00, bfbfbf, '
           'cfcfcf, ff9933, ccccff, bb0000, f5e9d5; dialogue 800000/000000/0000ff; values eaeaea; real panels use '
           'others too, e.g. ffb83f)')
    bad_sh = []
    for i in tw:
        w = S.byid[i]
        light = [c for _, c in R.colour_runs(w) if c not in DARK_TEXT]
        dark = [c for _, c in R.colour_runs(w) if c in DARK_TEXT]
        if light and not w['text_shadow']:
            bad_sh.append((i, 'light text without shadow'))
        if dark and not light and w['text_shadow']:
            bad_sh.append((i, 'dark text with shadow'))
    L.add('shadow rule (light text shadowed, dark text on parchment not)', f'{len(bad_sh)} of {len(tw)} break it',
          'PASS = 0 (99% of ff981f/ffffff/ff0000 text has shadow), BORDERLINE otherwise (1% exceptions)',
          'PASS' if not bad_sh else 'BORDERLINE')
    if overlay:
        hud_checks(L, comps, S, regions, texts)
    else:
        panel_checks(L, comps, S, regions, texts)
    v = L.verdict()
    return v, L.rows


def panel_checks(L, comps, S, regions, texts):
    pr = S.panel_rect()
    frames = [r for r in regions if r['family'] == 'steel' and r.get('divider') is not None]
    outer = next((r for r in frames if r['rect'] == pr), None)
    stone = next((r for r in regions if r['family'] in ('stone', 'stone2') and r['rect'] == pr), None)
    L.add('frame: titled steel border (2.1) around the whole panel', 'yes' if outer else 'stone border' if stone else 'no',
          'PASS = steel 2.1 (126 groups), BORDERLINE = stone 2.3 (36 groups)',
          'PASS' if outer else 'BORDERLINE' if stone else 'FAIL')
    if not outer:
        return
    fx, fy, W, H = outer['rect']
    canon = steel_227(W, H)
    graphics = [i for i in S.drawable() if S.byid[i]['kind'] == 5]
    placed = 0
    near2 = 0
    for s, (x, y, w, h) in canon:
        rect = [fx + x, fy + y, w, h]
        cands = [S.lay[i]['abs'] + S.lay[i]['size'] for i in graphics if S.byid[i]['sprites'][0] == s]
        if rect in cands:
            placed += 1
        elif any(near(rect, c) for c in cands):
            near2 += 1
    L.add('frame pieces placed as script 227 places them', f'{placed} of {len(canon)} exact, {near2} within 2px',
          'PASS = all 11 exact, BORDERLINE = all within 2px', 'PASS' if placed == len(canon) else
          'BORDERLINE' if placed + near2 == len(canon) else 'FAIL')
    titles = [t for t in texts if t['role'] == 'title' and R._inside(t['ink'], outer['rect'])]
    if not titles:
        L.add('title', 'none', 'PASS = a title in the title bar (every 2.1 panel has one)', 'FAIL')
    else:
        w = S.byid[titles[0]['id']]
        p = S.lay[titles[0]['id']]
        L.add('title font', MAIN_FONTS.get(w['font'], w['font']), 'PASS = b12_full (the title font throughout)',
              'PASS' if w['font'] == 496 else 'FAIL')
        c = w['colors'][0]
        L.add('title colour', f'{c:06x}', 'PASS = ff981f (187 frame titles), BORDERLINE = ff9933 (70)',
              'PASS' if c == 0xff981f else 'BORDERLINE' if c == 0xff9933 else 'FAIL')
        L.add('title shadow', bool(w['text_shadow']), 'PASS = on', 'PASS' if w['text_shadow'] else 'FAIL')
        box = [p['abs'][0] - fx, p['abs'][1] - fy] + p['size']
        al = w['text_alignment']
        L.add('title box and alignment', f'box {box}, align {al}', f'PASS = [6, 6, {W - 12}, 24] centred both ways '
              '(script 227), BORDERLINE = same letters within 2px', 'PASS' if box == [6, 6, W - 12, 24] and al == [1, 1]
              else 'BORDERLINE' if al[0] == 1 else 'FAIL')
    # content margins
    members = set(outer['members']) | {t['id'] for t in titles}
    content = [i for i in S.drawable() if i not in members]
    if content:
        boxes = []
        for i in content:
            cl = S.lay[i]['clip']
            boxes.append([cl[0], cl[1], cl[2] - cl[0], cl[3] - cl[1]])
        u = R._union(boxes)
        m = [u[0] - fx, u[1] - fy, fx + W - (u[0] + u[2]), fy + H - (u[1] + u[3])]
        L.info('content margins left, top, right, bottom', m,
               'INFO: (10, 40, 10, 10) exactly in 38 of 73 measured panels, within 5px in 47; others flush '
               '(0, 40, 0, 0) or one list (7, 7, 10, 10)')
    # buttons
    btns = [r for r in regions if r.get('button')]
    bad = []
    for r in btns:
        inside = [t for t in texts if R._inside(t['ink'], r['rect'])]
        lab = [S.byid[t['id']] for t in inside]
        ok_frame = r['family'] in R.BUTTONISH and r['family'] != 'sprite-built'
        ok_lab = all(w['font'] in (495, 494) and w['text_alignment'] == [1, 1] and w['text_shadow'] for w in lab)
        if not (ok_frame and ok_lab):
            bad.append(r['words'])
    if btns:
        L.add('buttons: 9px bevel box with a centred p12 (or p11) label, shadow', f'{len(btns) - len(bad)} of {len(btns)}',
              'PASS = all (2.4; 3746\'s buttons), BORDERLINE otherwise (real panels have other button art)',
              'PASS' if not bad else 'BORDERLINE')
    else:
        L.info('buttons', 'none found (a button = a bevel box with a text inside)', 'INFO')
    grid_checks(L, comps, S)
    # size band
    L.info('panel size against real 2.1 panels', f'{W}x{H}',
           'INFO: width 188-745, median 484 (quartiles 380, 500); height 133-503, median 308 (280, 325); commonest '
           '484x304, 480x314, 512x334, 488x300, 342x228, 500x324')
    # inner boxes from the standard set
    inner = [r for r in regions if r is not outer and r['family'] in ('steel', 'stone', 'stone2', 'bevel-dark',
                                                                      'bevel-light', 'thin', 'two-tone', 'sprite-built')]
    nonstd = [r for r in inner if r['family'] == 'sprite-built']
    if inner:
        L.add('inner boxes from the standard set (2.2, 2.4, 2.5, 2.6)', f'{len(inner) - len(nonstd)} of {len(inner)}',
              'PASS = all, BORDERLINE otherwise', 'PASS' if not nonstd else 'BORDERLINE')


def grid_checks(L, comps, S):
    """Item grids: declared ("grids": [{"layer": file, "columns": c, "rows": r}] in the JSON) or slot sprites/items
    laid out in rows and columns. Rule (INTERFACES.md 6): 36x32 slots, first flush top-left, last flush
    bottom-right: pitch = (W - 36) / (c - 1), (H - 32) / (r - 1)."""
    found = False
    for g in getattr(S, 'grids', None) or []:
        found = True
        lid = next((c['id'] for c in comps if c['file'] == g['layer']), None)
        if lid is None or lid not in S.lay:
            L.add('declared item grid', f'layer {g["layer"]} not found', 'PASS = an existing layer', 'FAIL')
            continue
        Wl, Hl = S.lay[lid]['size']
        c, r = g['columns'], g['rows']
        px_ = (Wl - 36) / (c - 1) if c > 1 else None
        py_ = (Hl - 32) / (r - 1) if r > 1 else None
        whole = all(v is None or float(v).is_integer() for v in (px_, py_))
        apart = all(v is None or v >= m for v, m in ((px_, 36), (py_, 32)))
        L.add('declared item grid (inv_init layer)', f'{c}x{r} in {Wl}x{Hl}: pitch {px_} x {py_}',
              'PASS = whole-pixel pitch, cells do not overlap (pitch >= 36x32), BORDERLINE otherwise '
              '(measured: 190x261 4x7 -> 51x38; 143x75 3x2 -> 53x43)', 'PASS' if whole and apart else 'BORDERLINE')
    slots = [i for i in S.drawable() if S.byid[i].get('item', -1) != -1 or
             (S.byid[i]['kind'] == 5 and S.byid[i]['sprites'][0] == 170)]
    byp = collections.defaultdict(list)
    for i in slots:
        byp[S.lay[i]['parent']].append(i)
    for p, ids in byp.items():
        xs = sorted({S.lay[i]['abs'][0] for i in ids}); ys = sorted({S.lay[i]['abs'][1] for i in ids})
        if len(xs) < 2 and len(ys) < 2:
            continue
        found = True
        lay = S.lay[p]
        Wl, Hl = lay['size']
        want_x = (Wl - 36) / (len(xs) - 1) if len(xs) > 1 else None
        want_y = (Hl - 32) / (len(ys) - 1) if len(ys) > 1 else None
        got_x = [b - a for a, b in zip(xs, xs[1:])]; got_y = [b - a for a, b in zip(ys, ys[1:])]
        ok = all(abs(g - want_x) <= 1 for g in got_x) if want_x else True
        ok &= all(abs(g - want_y) <= 1 for g in got_y) if want_y else True
        L.add('item grid pitch (inv_init rule)', f'{len(xs)}x{len(ys)} in {Wl}x{Hl}: pitch {got_x[:1]} {got_y[:1]}, rule '
              f'{want_x and round(want_x, 1)} {want_y and round(want_y, 1)}', 'PASS = within 1px of the rule, '
              'BORDERLINE otherwise (measured: 190x261 4x7 -> 51x38; 143x75 3x2 -> 53x43)',
              'PASS' if ok else 'BORDERLINE')
    if not found:
        L.info('item grid', 'none found', 'INFO')


def hud_checks(L, comps, S, regions, texts):
    root = next(c for c in comps if c['parent'] is None)
    fill = root['size_mode'] == [1, 1]
    anch = any(c['position_mode'] != [0, 0] for c in comps)
    L.add('overlay root fills its pane; content anchored by position modes', f'root size mode {root["size_mode"]}, '
          f'position modes used: {anch}', 'PASS = fill-parent root (1,1) (all 81 HUD/overlay groups), BORDERLINE '
          'otherwise', 'PASS' if fill else 'BORDERLINE')
    pr = S.panel_rect()
    L.info('drawn size against the real boss HUDs', f'{pr[2]}x{pr[3]} at {pr[:2]}',
           'INFO: 303 150x42 (242) / 300x42 (239); INFERNO_HP_HUD 596 300 wide; NIGHTMARE_TOTEMS 413 139x64; TOB_HUD 28 '
           '120x106')
    outl = [i for i in S.drawable() if S.byid[i]['kind'] == 3 and not S.byid[i]['filled']]
    wells = [i for i in S.drawable() if S.byid[i]['kind'] == 5 and S.byid[i]['sprites'][0] == 897]
    ok = False
    for o in outl:
        ro = S.lay[o]['abs'] + S.lay[o]['size']
        for w in wells:
            rw = S.lay[w]['abs'] + S.lay[w]['size']
            if rw[0] - ro[0] in (1, 2) and rw[1] - ro[1] in (1, 2) and S.byid[o]['colors'][0] == 0x191511:
                ok = True
    L.add('box: 1px 191511 frame around TRADEBACKING_DARK 897', 'yes' if ok else 'no',
          'PASS = as 303 (frame rectangle 191511, dark backing inset), BORDERLINE otherwise', 'PASS' if ok else 'BORDERLINE')
    fills = [i for i in S.drawable() if S.byid[i]['kind'] == 3 and S.byid[i]['filled'] and S.byid[i]['transparency'] == 0]
    reds = [i for i in fills if (S.byid[i]['colors'][0] >> 16) >= 0xa0 and (S.byid[i]['colors'][0] & 0xffff) == 0]
    greens = [i for i in fills if (S.byid[i]['colors'][0] & 0xff00ff) == 0 and (S.byid[i]['colors'][0] >> 8) & 255 >= 0x80]
    bar = False
    for r in reds:
        rr = S.lay[r]['abs'] + S.lay[r]['size']
        for g in greens:
            rg = S.lay[g]['abs'] + S.lay[g]['size']
            if rg[1] == rr[1] and rg[3] == rr[3] and rg[0] == rr[0] and S.order.index(g) > S.order.index(r):
                bar = True
    L.add('bar: red under green, green from the left', 'yes' if bar else 'no',
          'PASS = as 303/596 (cc0000 or c80000 under 009500/00cc00/00c800)', 'PASS' if bar else 'FAIL')
    hs = sorted({S.lay[i]['size'][1] for i in fills})
    L.add('bar heights', hs, 'PASS = 9-21px (the bars of the 81 HUD/overlay groups), BORDERLINE otherwise',
          'PASS' if hs and all(9 <= h <= 21 for h in hs) else 'BORDERLINE')
    tw = [S.byid[t['id']] for t in texts]
    name = next((t for t in texts if t['role'] == 'label'), None)
    if name:
        w = S.byid[name['id']]
        ok = w['font'] == 494 and w['colors'][0] == 0xff981f and w['text_shadow'] and w['text_alignment'][0] == 1
        L.add('name: p11_full ff981f shadow centred', f'{MAIN_FONTS.get(w["font"], w["font"])} {w["colors"][0]:06x} '
              f'shadow {bool(w["text_shadow"])} align {w["text_alignment"]}', 'PASS = as 303 and 596',
              'PASS' if ok else 'BORDERLINE')
    else:
        L.add('name', 'none', 'PASS = a name label above the bar', 'BORDERLINE')
    val = [t for t in texts if t['role'] == 'value']
    if val:
        w = S.byid[val[0]['id']]
        ok = w['font'] == 494 and w['colors'][0] in (0xeaeaea, 0xffffff, 0x000000) and w['text_alignment'][0] == 1
        L.add('value: p11_full eaeaea centred over the bar', f'{MAIN_FONTS.get(w["font"], w["font"])} '
              f'{w["colors"][0]:06x} align {w["text_alignment"]}', 'PASS = as 303 (596 uses black)',
              'PASS' if ok else 'BORDERLINE')
    else:
        L.add('value', 'none', 'PASS = a value centred over the bar', 'BORDERLINE')


# ---------------------------------------------------------------- draw

def draw_cmd(comps, out, pane, scale):
    comps = R.normalize(comps)
    S = R.rebuild_scene(comps, pane)
    a, _, rep = S.images()
    Image.fromarray(a.astype(np.uint8)).save(out)
    x, y, w, h = S.panel_rect()
    if w and h:
        crop = a[y:y + h, x:x + w].astype(np.uint8)
        stem = str(Path(out).with_suffix(''))
        Image.fromarray(crop).resize((w * scale, h * scale), Image.NEAREST).save(f'{stem}-crop{scale}x.png')
    print(f'drawn {len(rep["drawn"])}, not drawn {len(rep["not_drawn"])} (models/items), covered {w}x{h} at ({x},{y}) '
          f'of the {S.pane[0]}x{S.pane[1]} pane')


def main():
    ap = argparse.ArgumentParser()
    sub = ap.add_subparsers(dest='cmd', required=True)
    a1 = sub.add_parser('rebuild'); a1.add_argument('group', type=int); a1.add_argument('json')
    a1.add_argument('--detail', action='store_true'); a1.add_argument('--out')
    a2 = sub.add_parser('original'); a2.add_argument('json'); a2.add_argument('--out')
    a3 = sub.add_parser('draw'); a3.add_argument('json'); a3.add_argument('out')
    a3.add_argument('--pane', choices=['modal', 'hud']); a3.add_argument('--scale', type=int, default=3)
    a = ap.parse_args()
    comps, meta = R.load_components(a.json)
    if a.cmd == 'draw':
        draw_cmd(comps, a.out, a.pane or ('hud' if meta.get('overlay') or meta.get('pane') == 'hud' else 'modal'), a.scale)
        return
    stem = Path(a.json).stem
    if a.cmd == 'rebuild':
        if a.detail:
            out = Path(a.out) if a.out else Path(a.json).parent / f'gate-{stem}'
            out.mkdir(parents=True, exist_ok=True)
            bp = out / 'board.png'
        else:
            out = Path(a.out) if a.out else None
            bdir = R.HERE / 'sheets-teacher-only' / str(a.group) / 'gate-boards'
            bdir.mkdir(parents=True, exist_ok=True)
            bp = bdir / f'{stem}-board.png'
        v, rows, px = gate_rebuild(a.group, comps, a.detail, out, board_path=bp)
        if out:
            out.mkdir(parents=True, exist_ok=True)
            (out / 'gate.json').write_text(json.dumps(dict(verdict=v, lines=rows), indent=1))
    else:
        v, rows = gate_original(comps, bool(meta.get('overlay')), grids=meta.get('grids'))
        out = Path(a.out) if a.out else Path(a.json).parent / f'gate-{stem}'
        out.mkdir(parents=True, exist_ok=True)
        (out / 'gate.json').write_text(json.dumps(dict(verdict=v, lines=rows), indent=1))


if __name__ == '__main__':
    main()
