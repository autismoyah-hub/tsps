"""ifref.py: shared machinery for ifsheet.py and ifgate.py (Teacher side).

A SCENE is a laid-out component tree in PANE coordinates (the pane the panel opens into: the 512x334 main modal, or
the HUD pane as TSPS gave it), ready for iface.draw: (order, lay, byid, pane size).
- real_scene(group): the real panel as TSPS drew it (the tooling client's state after onLoad, or TSPS's own
  runtime values for the HUD), only the panel's own subtree, shifted into the pane. Teacher data.
- rebuild_scene(components, pane): an author's component list (the proof/rebuild-148.json form), laid out by
  iface.layout in the pane.
- classify(scene): the deterministic wire procedure (regions with roles in words, texts with roles), used by
  ifsheet.py for the sheet, by ifgate.py for the damage tests and the ORIGINAL checks.

All drawing uses cache 242 (what TSPS runs) and TSPS's WebGL blend (iface.TSPS_BLEND = True).
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, json, re
from pathlib import Path
import numpy as np

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import ifcache as C
import iface
import ifpalette as PAL

iface.TSPS_BLEND = True

PANES = {'modal': (512, 334), 'hud': (1230, 708)}

# The targets of NEXT.md section 1 (each exact against TSPS: tsps/modal/compare.json, tsps/hud2) plus 148 (proof).
TARGETS = {
    68: dict(name='SLAYER_PARTNER', state='state_modal/68.jsonl', pane='modal', tsps='tsps/modal/cmp/68-tsps.png'),
    648: dict(name='TARGET_STREAKS', state='state_modal/648.jsonl', pane='modal', tsps='tsps/modal/cmp/648-tsps.png'),
    210: dict(name='POH_PETLIST', state='state_modal/210.jsonl', pane='modal', tsps='tsps/modal/cmp/210-tsps.png'),
    912: dict(name='POH_TROPHY_MENU', state='state_modal/912.jsonl', pane='modal', tsps='tsps/modal/cmp/912-tsps.png'),
    476: dict(name='BR_SETTINGS', state='state_modal/476.jsonl', pane='modal', tsps='tsps/modal/cmp/476-tsps.png'),
    383: dict(name='GE_HISTORY', state='state_modal/383.jsonl', pane='modal', tsps='tsps/modal/cmp/383-tsps.png'),
    303: dict(name='HPBAR_HUD', state='state242/tsps-303-hud5.jsonl', pane='hud', tsps='tsps/hud2/cmp-303-hud5-tsps.png'),
    148: dict(name='PET_INSURANCE', state='state_modal/148.jsonl', pane='modal', tsps='tsps/modal/cmp/148-tsps.png'),
}

_IFNAMES = None


def if_name(group):
    global _IFNAMES
    if _IFNAMES is None:
        _IFNAMES = {}
        p = Path(_AP.fix(r'<TSPS_CHECKOUT>/client/runelite/api/gameval/InterfaceID.ts'))
        if p.exists():
            _IFNAMES = {int(i): n for n, i in re.findall(r'export const (\w+) = (\d+);', p.read_text())}
    return _IFNAMES.get(group, f'GROUP_{group}')


def exact_pool():
    """Modal panels whose iface.py drawing of the captured state equals the TSPS capture over every comparable pixel
    (tsps/modal/compare.json, differ == 0): the panels a sheet and a control can be made for."""
    d = json.loads((HERE / 'tsps' / 'modal' / 'compare.json').read_text())
    return sorted(x['group'] for x in d if x.get('differ') == 0)


def ensure_target(group):
    """Register a pool panel (exact against TSPS) as a target with the standard modal paths (v2, held-out tests)."""
    if group in TARGETS:
        return TARGETS[group]
    if group not in exact_pool():
        raise KeyError(f'{group} is not exact against TSPS (tsps/modal/compare.json); no trusted reference')
    TARGETS[group] = dict(name=if_name(group), state=f'state_modal/{group}.jsonl', pane='modal',
                          tsps=f'tsps/modal/cmp/{group}-tsps.png')
    return TARGETS[group]

# Sprite families (INTERFACES.md section 2). Corner order: top-left, top-right, bottom-left, bottom-right.
FAMILIES = {
    'steel': dict(corners=(310, 311, 312, 313), edges={172, 173, 314, 315}),
    'stone': dict(corners=(824, 825, 826, 827), edges={820, 821, 822, 823}),
    'stone2': dict(corners=(846, 847, 848, 849), edges={828, 841, 820, 821, 822, 823}),
    'bevel-dark': dict(corners=(913, 914, 915, 916), edges={917, 918, 919, 920}),
    'bevel-light': dict(corners=(929, 930, 931, 932), edges={933, 934, 935, 936}),
    'thin': dict(corners=(991, 992, 993, 994), edges={987, 988, 989, 990}),
    # v2 (INTERFACES.md section 2): families found by the census of all groups
    'bevel-mid': dict(corners=(921, 922, 923, 924), edges={925, 926, 927, 928}),
    'bevel-beige': dict(corners=(1554, 1555, 1556, 1557), edges={1558, 1559, 1560, 1561}),
    'tile-grey': dict(corners=(1141, 1143, 1147, 1149), edges={1142, 1144, 1145, 1146, 1148}),
    'tile-red': dict(corners=(1150, 1152, 1156, 1158), edges={1151, 1153, 1154, 1155, 1157}),
}
FAMILY_WORDS = {'bevel-dark': 'dark bevel box', 'bevel-light': 'light bevel box', 'bevel-mid': 'medium bevel box',
                'bevel-beige': 'beige bevel box', 'tile-grey': 'grey tile box', 'tile-red': 'red tile box'}
BUTTONISH = ('bevel-dark', 'bevel-light', 'bevel-mid', 'bevel-beige', 'tile-grey', 'tile-red', 'sprite-built')
DIVIDERS = {2546, 828, 829, 830, 995, 996, 997}
CLOSES = {535, 536, 831, 539, 541, 537}
BORDERS = set(DIVIDERS) | CLOSES
for _f in FAMILIES.values():
    BORDERS |= set(_f['corners']) | _f['edges']
BACKINGS = {297: 'backing', 1040: 'light well', 897: 'dark well'}

_cache = None


def cache():
    global _cache
    if _cache is None:
        _cache = C.Cache(C.CACHE_242)
    return _cache


_icons = None


def icons():
    global _icons
    if _icons is None:
        _icons = iface.text_icons(cache())
    return _icons


class Scene:
    def __init__(self, order, lay, byid, pane, kind):
        self.order, self.lay, self.byid, self.pane, self.kind = order, lay, byid, tuple(pane), kind
        self._ink = {}
        self._img = None

    # ------------------------------------------------------------ drawing
    def draw(self, bg=0x29251f, only=None):
        w, h = self.pane
        ras = iface.Raster(w, h, 0xff000000 | bg)
        order = self.order if only is None else only
        ras, rep = iface.draw(cache(), order, self.lay, self.byid, ras, (w, h))
        return ras.rgb().astype(np.int16), rep

    def images(self):
        """(drawing on dark bg, drawing on bright green bg, report): both needed to see where the panel covers."""
        if self._img is None:
            a, rep = self.draw(0x29251f)
            b, _ = self.draw(0x00ff00)
            self._img = (a, b, rep)
        return self._img

    def covered(self):
        a, b, _ = self.images()
        bga = np.array([0x29, 0x25, 0x1f]); bgb = np.array([0, 255, 0])
        return (a != bga).any(-1) | (b != bgb).any(-1)

    def panel_rect(self):
        """Bounding box [x, y, w, h] of the pixels the panel covers in the pane."""
        m = self.covered()
        ys, xs = np.nonzero(m)
        if not len(xs):
            return [0, 0, 0, 0]
        return [int(xs.min()), int(ys.min()), int(xs.max() - xs.min() + 1), int(ys.max() - ys.min() + 1)]

    def excluded(self):
        """Mask of model and item-icon rectangles (iface.py does not draw them)."""
        _, _, rep = self.images()
        w, h = self.pane
        m = np.zeros((h, w), bool)
        rects = []
        for i, why in rep['not_drawn']:
            if why.startswith('special content') or why.startswith('if1 tooltip'):
                continue
            cl = self.lay[i]['clip']
            x0, y0, x1, y1 = max(cl[0], 0), max(cl[1], 0), max(cl[2], 0), max(cl[3], 0)
            m[y0:y1, x0:x1] = True
            rects.append([i, why, [x0, y0, x1 - x0, y1 - y0]])
        return m, rects

    def ink(self, i):
        """(mask, bbox [x,y,w,h] or None) of what component i draws on its own (clipped as in the panel)."""
        if i not in self._ink:
            a, _ = self.draw(0x000000, only=[i])
            b, _ = self.draw(0xffffff, only=[i])
            m = (a != 0).any(-1) | (b != 255).any(-1)
            ys, xs = np.nonzero(m)
            box = None if not len(xs) else [int(xs.min()), int(ys.min()), int(xs.max() - xs.min() + 1),
                                             int(ys.max() - ys.min() + 1)]
            self._ink[i] = (m if box else None, box)
        return self._ink[i]

    def drawable(self):
        """Visible components that put something on the screen (layers excluded), in draw order. v2: a rectangle
        or line entirely covered by a later opaque filled rectangle (it changes no pixel of the drawing, e.g. the
        dark green under the light green of 303's bar) is not drawable: no sheet can show it, no gate scores it."""
        if getattr(self, '_drawable', None) is not None:
            return list(self._drawable)
        out = []
        for i in self.order:
            w, p = self.byid[i], self.lay[i]
            if p['hidden'] or w['kind'] in (0, 11):
                continue
            if w['kind'] == 3 and w['transparency'] >= 255 and w.get('fill_mode', 0) == 0:
                continue
            if w['kind'] == 6 or (w.get('item', -1) != -1):
                out.append(i); continue
            if self.ink(i)[1] is not None:
                out.append(i)
        out = [i for i in out if not self.occluded(i, out)]
        self._drawable = out
        return list(out)

    def occluded(self, i, drawn):
        w = self.byid[i]
        if w['kind'] not in (3, 9):
            return False
        b = self.ink(i)[1]
        pos = self.order.index(i)
        cover = None
        for j in drawn:
            if self.order.index(j) <= pos:
                continue
            u = self.byid[j]
            if u['kind'] == 3 and u['filled'] and u['transparency'] == 0 and u.get('fill_mode', 0) == 0:
                c = self.ink(j)[1]
                if c and c[0] <= b[0] and c[1] <= b[1] and c[0] + c[2] >= b[0] + b[2] and c[1] + c[3] >= b[1] + b[3]:
                    cover = j
                    break
        if cover is None:
            return False
        # confirm: the drawing without i is identical
        full = self.draw(0x29251f)[0]
        rest = self.draw(0x29251f, only=[k for k in self.order if k != i])[0]
        return bool((full == rest).all())

    def is_ancestor(self, a, i):
        p = self.lay[i]['parent']
        while p is not None:
            if p == a:
                return True
            p = self.lay[p]['parent']
        return False


# ---------------------------------------------------------------- the real panel

def real_scene(group):
    t = ensure_target(group)
    order, lay, byid = iface.load_state(HERE / t['state'])
    root = group << 16
    sub = [i for i in order if i == root or _descends(lay, i, root)]
    par = lay[root]['parent']
    if t['pane'] == 'hud' or par is None:
        pane = [0, 0] + list(lay[root]['size'])
    else:
        pane = list(lay[par]['abs']) + list(lay[par]['size'])
    if t['pane'] == 'modal':
        assert tuple(pane[2:]) == PANES['modal'], pane
    dx, dy = pane[0], pane[1]
    nlay, nby = {}, {}
    for i in sub:
        p = dict(lay[i])
        p['abs'] = [p['abs'][0] - dx, p['abs'][1] - dy]
        c = p['clip']
        p['clip'] = [c[0] - dx, c[1] - dy, c[2] - dx, c[3] - dy]
        if i == root:
            p['parent'] = None
        nlay[i] = p
        nby[i] = byid[i]
    return Scene(sub, nlay, nby, pane[2:], t['pane'])


def _descends(lay, i, root):
    p = lay[i]['parent']
    while p is not None:
        if p == root:
            return True
        p = lay.get(p, {}).get('parent')
    return False


# ---------------------------------------------------------------- component lists

TEMPLATE = dict(if3=True, kind=0, button=0, content=0, raw_position=[0, 0], raw_size=[0, 0], size_mode=[0, 0],
                position_mode=[0, 0], parent=None, hidden=False, mouse_over=-1, scroll_size=[0, 0],
                no_click_through=False, colors=[0, 0, 0, 0], filled=False, transparency=0, transparency_bottom=0,
                fill_mode=0, line_width=1, line_reversed=False, sprites=[-1, -1], sprite_angle=0, sprite_tiling=False,
                outline=0, sprite_shadow=0, flip_v=False, flip_h=False, models=[-1, -1], sequences=[-1, -1],
                model_offset=[0, 0], model_angles=[0, 0, 0], model_zoom=100, model_ortho=False, model_extra=0,
                model_scale=[0, 0], font=-1, text='', alternate_text='', text_alignment=[0, 0], line_height=0,
                text_shadow=False, flags=0, data_text='', actions=[], drag_zone=0, drag_threshold=0, scrollbar=False,
                spell_action='', spell_name='', button_text='Ok', listeners={}, triggers={}, cs1_comparisons=[], cs1=[],
                item=-1, count=0)


def load_components(path):
    spec = json.loads(Path(path).read_text())
    comps = spec['components'] if isinstance(spec, dict) else spec
    meta = spec if isinstance(spec, dict) else {}
    return comps, meta


def normalize(comps, group=None):
    """Fill defaults; ids from group and file; a parent may be given as a file number or a full id."""
    if group is None:
        group = next((c['group'] for c in comps if 'group' in c), None)
        if group is None:
            group = next(((c['id'] >> 16) for c in comps if 'id' in c), 30900)
    out = []
    for n, c in enumerate(comps):
        w = json.loads(json.dumps(TEMPLATE))
        w.update(c)
        f = w.get('file', n)
        w['file'], w['group'], w['id'] = f, group, (group << 16) | f
        p = w.get('parent')
        if p is not None and p >= 0 and p < 65536:
            w['parent'] = (group << 16) | p
        elif p is not None and p >= 65536:
            w['parent'] = (group << 16) | (p & 0xffff)
        elif p is not None and p < 0:
            w['parent'] = None
        w['colors'] = (list(w['colors']) + [0, 0, 0, 0])[:4]
        if isinstance(w['sprites'], int):
            w['sprites'] = [w['sprites'], -1]
        out.append(w)
    return out


def rebuild_scene(comps, pane='modal'):
    size = PANES[pane] if isinstance(pane, str) else tuple(pane)
    lay, order = iface.layout(comps, size)
    byid = {w['id']: w for w in comps}
    return Scene(order, lay, byid, size, pane if isinstance(pane, str) else 'modal')


def scene_to_components(scene, group):
    """A component list that iface.layout puts exactly where the scene has its components (the CONTROL form)."""
    index = {i: n for n, i in enumerate(scene.order)}
    out = []
    for n, i in enumerate(scene.order):
        w, p = scene.byid[i], scene.lay[i]
        c = {k: v for k, v in w.items() if k in TEMPLATE}
        par = p['parent']
        pab = scene.lay[par]['abs'] if par is not None else [0, 0]
        c.update(file=n, group=group, id=(group << 16) | n, parent=None if par is None else (group << 16) | index[par],
                 raw_position=[p['abs'][0] - pab[0], p['abs'][1] - pab[1]], raw_size=list(p['size']),
                 size_mode=[0, 0], position_mode=[0, 0], hidden=bool(p['hidden']))
        c.pop('listeners', None); c.pop('triggers', None); c.pop('cs1', None)
        out.append(c)
    return normalize(out, group)


# ---------------------------------------------------------------- text helpers

def visible_text(w):
    t = w['alternate_text'] if (w.get('condition') and w.get('alternate_text')) else w['text']
    return t if w['if3'] else iface.substitute(t)


def strip_tags(s):
    s = re.sub(r'<br>', '\n', s)
    s = re.sub(r'<lt>', '<', s); s = re.sub(r'<gt>', '>', s)
    return re.sub(r'<[^>]*>', '', s)


def plain(s):
    return ' '.join(strip_tags(s).split())


def colour_runs(w):
    """Per inked glyph: (glyph byte, colour) after <col> markup; the text's colour as the eye sees it."""
    base = w['colors'][0]
    col, out = base, []
    for kind, v in iface.tokens(visible_text(w)):
        if kind == 't':
            if v.startswith('col='):
                try:
                    col = int(v[4:], 16)
                except ValueError:
                    pass
            elif v in ('/col', 'br'):
                col = base
        elif v != 32:
            out.append((v, col))
    return out


def text_lines(scene, i):
    w, p = scene.byid[i], scene.lay[i]
    fnt = C.font(cache(), w['font'])
    lh = w['line_height'] or fnt.metrics.ascent
    width, height = p['size']
    limits = None if (height < fnt.max_ascent + fnt.max_descent + lh and height < lh * 2) else [width]
    return [plain(l) for l in iface.wrap(fnt.metrics, visible_text(w), limits, icons())]


def emphasised(w, named=False):
    """Parts of a text drawn in another colour than the text's own (named as parts; with named=True each part is
    followed by its colour's name in the INTERFACES.md palette, e.g. 'Total: (light orange)')."""
    base = w['colors'][0]
    parts, cur, col = [], '', base
    if named:
        return [f'{p} ({PAL.name(c)})' for p, c in _emph_runs(w)]
    for kind, v in iface.tokens(visible_text(w)):
        if kind == 't':
            nc = col
            if v.startswith('col='):
                try:
                    nc = int(v[4:], 16)
                except ValueError:
                    pass
            elif v in ('/col', 'br'):
                nc = base
            if nc != col:
                if col != base and cur.strip():
                    parts.append(cur.strip())
                cur, col = '', nc
            elif v == 'br':
                cur += ' '
        else:
            cur += chr(v) if v < 128 else bytes([v]).decode('cp1252', 'replace')
    if col != base and cur.strip():
        parts.append(cur.strip())
    return parts


def _emph_runs(w):
    """[(text, colour)] of the parts of a text drawn in a colour other than its base colour."""
    base = w['colors'][0]
    out, cur, col = [], '', base
    for kind, v in iface.tokens(visible_text(w)):
        if kind == 't':
            nc = col
            if v.startswith('col='):
                try:
                    nc = int(v[4:], 16)
                except ValueError:
                    pass
            elif v in ('/col', 'br'):
                nc = base
            if nc != col:
                if col != base and cur.strip():
                    out.append((cur.strip(), col))
                cur, col = '', nc
            elif v == 'br':
                cur += ' '
        else:
            cur += chr(v) if v < 128 else bytes([v]).decode('cp1252', 'replace')
    if col != base and cur.strip():
        out.append((cur.strip(), col))
    return out


VALUE_RE = re.compile(r'^[\d\s,./%()+\-:kKmMxX]*\d[\d\s,./%()+\-:kKmMxX]*$')


# ---------------------------------------------------------------- the wire procedure

def _inside(a, b, tol=1):
    """box a inside box b (x,y,w,h)."""
    return (a[0] >= b[0] - tol and a[1] >= b[1] - tol and a[0] + a[2] <= b[0] + b[2] + tol
            and a[1] + a[3] <= b[1] + b[3] + tol)


def _union(boxes):
    x0 = min(b[0] for b in boxes); y0 = min(b[1] for b in boxes)
    x1 = max(b[0] + b[2] for b in boxes); y1 = max(b[1] + b[3] for b in boxes)
    return [x0, y0, x1 - x0, y1 - y0]


def classify(scene):
    """Deterministic: regions (role in words, rectangle in pane coordinates, member components) and texts (role,
    lines, ink box), from the components the panel draws. Uses sprite FAMILIES only to name roles; no sprite id,
    font or colour is written into the result."""
    S = scene
    draw = S.drawable()
    byid, lay = S.byid, S.lay
    used = set()
    regions = []

    def spr(i):
        w = byid[i]
        return w['sprites'][0] if w['kind'] == 5 else None

    graphics = [i for i in draw if byid[i]['kind'] == 5 and byid[i].get('item', -1) == -1]
    texts = [i for i in draw if byid[i]['kind'] == 4]
    text_slots = [i for i in S.order if byid[i]['kind'] == 4 and not lay[i]['hidden']]  # incl. empty ones

    # 1. framed boxes from corner families
    for fam, spec in FAMILIES.items():
        tl, tr, bl, br = spec['corners']
        for a in [i for i in graphics if spr(i) == tl and i not in used]:
            pa = lay[a]['parent']
            ax, ay = lay[a]['abs']
            best = None
            for d in [i for i in graphics if spr(i) == br and lay[i]['parent'] == pa and i not in used]:
                dx, dy = lay[d]['abs']
                if dx < ax or dy < ay:
                    continue
                b = next((i for i in graphics if spr(i) == tr and lay[i]['parent'] == pa and lay[i]['abs'] == [dx, ay]), None)
                c = next((i for i in graphics if spr(i) == bl and lay[i]['parent'] == pa and lay[i]['abs'] == [ax, dy]), None)
                if b is None or c is None:
                    continue
                area = (dx - ax) * (dy - ay)
                if best is None or area < best[0]:
                    best = (area, d, b, c)
            if best is None:
                continue
            _, d, b, c = best
            box = [ax, ay, lay[d]['abs'][0] + lay[d]['size'][0] - ax, lay[d]['abs'][1] + lay[d]['size'][1] - ay]
            members = [a, b, c, d]
            for e in graphics:
                if e in used or e in members or lay[e]['parent'] != pa:
                    continue
                if spr(e) in spec['edges'] and _inside(S.ink(e)[1], box):
                    members.append(e)
            filled = False
            fill_info = None
            for e in graphics:
                if e in used or e in members or lay[e]['parent'] != pa:
                    continue
                # the box's own backing: the box inset 1px (steel, stone, bevels) or the whole box (thin, v2)
                for d in (1, 0):
                    if spr(e) in BACKINGS and byid[e]['sprite_tiling'] and lay[e]['abs'] == [box[0] + d, box[1] + d] \
                            and lay[e]['size'] == [box[2] - 2 * d, box[3] - 2 * d]:
                        members.append(e); filled = spr(e)
                        fill_info = dict(sprite=spr(e), inset=d, transparency=byid[e]['transparency'],
                                         flip_h=bool(byid[e]['flip_h']), flip_v=bool(byid[e]['flip_v']))
                        break
            divider = close = None
            for e in graphics:
                if e in used or e in members or lay[e]['parent'] != pa or not _inside(S.ink(e)[1], box):
                    continue
                if spr(e) in DIVIDERS:
                    members.append(e); divider = divider or e
                elif spr(e) in CLOSES:
                    members.append(e); close = e
            used.update(members)
            band = None
            if fam == 'thin' and divider is not None:
                bands = [S.ink(m)[1] for m in members if spr(m) == 995 and S.ink(m)[1]]
                if bands:
                    b = bands[0]
                    band = [b[0] - box[0], b[1] - box[1], b[2], b[3]]
            regions.append(dict(family=fam, rect=box, members=members, filled=filled, divider=divider, close=close,
                                band=band, fill_info=fill_info))

    # 2. sprite-built boxes not in a family: a layer whose own (non-backing) sprites cover its four corners and
    #    at least 90% of it
    for L in [i for i in S.order if byid[i]['kind'] == 0 and not lay[i]['hidden']]:
        kids = [i for i in graphics if lay[i]['parent'] == L and i not in used and spr(i) not in BACKINGS]
        if len(kids) < 4:
            continue
        box = list(lay[L]['abs']) + list(lay[L]['size'])
        x, y, w, h = box
        if w <= 2 or h <= 2 or x < 0 or y < 0:
            continue
        m = np.zeros(S.pane[::-1], bool)
        for k in kids:
            kb = S.ink(k)[1]
            if kb is None:  # an item slot drawn as a graphic (not drawn by iface.py)
                continue
            m[kb[1]:kb[1] + kb[3], kb[0]:kb[0] + kb[2]] = True
        corners = [(y, x), (y, x + w - 1), (y + h - 1, x), (y + h - 1, x + w - 1)]
        if not all(0 <= cy < m.shape[0] and 0 <= cx < m.shape[1] and m[cy, cx] for cy, cx in corners):
            continue
        if m[y:y + h, x:x + w].mean() >= 0.9:
            used.update(kids)
            regions.append(dict(family='sprite-built', rect=box, members=list(kids), filled=None, divider=None,
                                close=None))

    # 3. close buttons, backings, icons, rectangles, lines, models, items
    for i in graphics:
        if i in used:
            continue
        s = spr(i); box = S.ink(i)[1]
        if s in CLOSES:
            regions.append(dict(family='close', rect=box, members=[i])); used.add(i)
        elif s in BACKINGS and byid[i]['sprite_tiling']:
            regions.append(dict(family='well', well=BACKINGS[s], translucent=byid[i]['transparency'] > 0,
                                transparency=byid[i]['transparency'], flip_h=bool(byid[i]['flip_h']),
                                flip_v=bool(byid[i]['flip_v']), rect=box, members=[i])); used.add(i)
        elif s in DIVIDERS:
            regions.append(dict(family='divider', rect=box, members=[i])); used.add(i)
    icon_ids = {}
    for i in graphics:
        if i in used:
            continue
        key = (spr(i), byid[i]['flip_h'], byid[i]['flip_v'], tuple(S.ink(i)[1][2:]))
        icon_ids.setdefault(key, chr(65 + len(icon_ids)) if len(icon_ids) < 26 else str(len(icon_ids)))
        regions.append(dict(family='icon', icon=icon_ids[key], rect=S.ink(i)[1], members=[i])); used.add(i)
    rects = [i for i in draw if byid[i]['kind'] == 3 and i not in used]
    outl = [i for i in rects if not byid[i]['filled']]
    for a in outl:
        if a in used:
            continue
        ra = list(lay[a]['abs']) + list(lay[a]['size'])
        inner = next((b for b in outl if b != a and b not in used and lay[b]['abs'] == [ra[0] + 1, ra[1] + 1]
                      and lay[b]['size'] == [ra[2] - 2, ra[3] - 2]), None)
        if inner is not None:
            regions.append(dict(family='two-tone', rect=ra, members=[a, inner], colours=[byid[a]['colors'][0],
                                byid[inner]['colors'][0]], transparency=byid[a]['transparency'])); used.update([a, inner])
        else:
            regions.append(dict(family='outline', rect=ra, members=[a], colours=[byid[a]['colors'][0]],
                                transparency=byid[a]['transparency'])); used.add(a)
    fills = [i for i in rects if byid[i]['filled'] and i not in used]
    bars = [i for i in fills if byid[i]['transparency'] == 0 and lay[i]['size'][1] <= 21]
    for i in [i for i in fills if i not in bars]:
        r = S.ink(i)[1]
        regions.append(dict(family='highlight' if byid[i]['transparency'] > 0 else 'filled', rect=r, members=[i],
                            colours=[byid[i]['colors'][0]], transparency=byid[i]['transparency']))
        used.add(i)
    while bars:
        a = max(bars, key=lambda i: lay[i]['size'][0])
        ra = list(lay[a]['abs']) + list(lay[a]['size'])
        same = [i for i in bars if lay[i]['abs'][1] == ra[1] and lay[i]['size'][1] == ra[3]
                and ra[0] <= lay[i]['abs'][0] and lay[i]['abs'][0] + lay[i]['size'][0] <= ra[0] + ra[2]]
        parts = sorted({tuple(S.ink(i)[1]) for i in same if i != a and lay[i]['size'][0] < ra[2] and S.ink(i)[1]})
        part_cols = [byid[i]['colors'][0] for i in sorted(same, key=S.order.index) if i != a]
        regions.append(dict(family='bar', rect=S.ink(a)[1] or ra, members=same, parts=[list(p) for p in parts],
                            colours=[byid[a]['colors'][0]] + part_cols))
        used.update(same)
        bars = [i for i in bars if i not in same]
    for i in draw:
        w = byid[i]
        if i in used:
            continue
        if w['kind'] == 9:
            regions.append(dict(family='line', rect=S.ink(i)[1], members=[i], colours=[w['colors'][0]],
                                width=w['line_width'])); used.add(i)
        elif w['kind'] == 6:
            cl = lay[i]['clip']
            regions.append(dict(family='model', rect=[cl[0], cl[1], cl[2] - cl[0], cl[3] - cl[1]], members=[i]))
            used.add(i)
        elif w.get('item', -1) != -1:
            cl = lay[i]['clip']
            regions.append(dict(family='item', rect=[cl[0], cl[1], cl[2] - cl[0], cl[3] - cl[1]], members=[i],
                                parent=lay[i]['parent']))
            used.add(i)
    # item grids: items sharing a parent
    items = [r for r in regions if r['family'] == 'item']
    byp = {}
    for r in items:
        byp.setdefault(r['parent'], []).append(r)
    for p, rs in byp.items():
        if len(rs) >= 2:
            cols = len({r['rect'][0] for r in rs}); rows = len({r['rect'][1] for r in rs})
            for r in rs:
                regions.remove(r)
            regions.append(dict(family='grid', grid=[cols, rows], rect=_union([r['rect'] for r in rs]),
                                members=sum([r['members'] for r in rs], [])))

    # buttons: a bevel or sprite-built box with a text component inside it
    for r in regions:
        if r['family'] in BUTTONISH:
            r['button'] = any(_inside(list(lay[t]['abs']) + list(lay[t]['size']), r['rect'], 0) for t in text_slots)

    # 4. texts with roles
    titled = [r for r in regions if r.get('divider') is not None]
    out_texts = []
    for i in texts:
        w = byid[i]
        box = S.ink(i)[1]
        lines = text_lines(S, i)
        s = plain(visible_text(w))
        role = None
        for r in regions:
            if r.get('button') and _inside(box, r['rect']):
                role = 'button'
        for r in titled:
            if role is not None:
                break
            dv = S.ink(r['divider'])[1]
            if _inside(box, r['rect']) and box[1] + box[3] <= dv[1] + 2:
                role = 'title' if r['family'] in ('steel', 'stone', 'stone2') else 'heading'
        if role is None:
            role = 'body' if len(s) > 40 else 'value' if VALUE_RE.match(s) else 'label'
        fnt = C.font(cache(), w['font'])
        pitch = (w['line_height'] or fnt.metrics.ascent) if len(lines) > 1 else None
        lines_aligned = ['left', 'centre', 'right'][min(w['text_alignment'][0], 2)] if len(lines) > 1 else None
        out_texts.append(dict(id=i, role=role, lines=lines, text=s, ink=box, emphasised=emphasised(w, named=True),
                              colour=PAL.name(w['colors'][0]), pitch=pitch, lines_aligned=lines_aligned))
    # reading order: rows by ink top (within 4px), then left to right
    out_texts.sort(key=lambda t: (t['ink'][1], t['ink'][0]))
    rows, cur = [], []
    for t in out_texts:
        if cur and t['ink'][1] - cur[0]['ink'][1] > 4:
            rows.append(cur); cur = []
        cur.append(t)
    if cur:
        rows.append(cur)
    out_texts = [t for row in rows for t in sorted(row, key=lambda t: t['ink'][0])]

    regions.sort(key=lambda r: (r['family'] == 'icon', -(r['rect'][2] * r['rect'][3]) if r['family'] != 'icon'
                                else 0, r['rect'][1], r['rect'][0]))
    letters = {}
    for r in regions:
        if r['family'] == 'icon':
            r['icon'] = letters.setdefault(r['icon'], chr(65 + len(letters)) if len(letters) < 26 else str(len(letters)))
        r['words'] = words(r)
    # v2: the order regions are drawn in (1 = first): by the first-drawn of their members. Where regions overlap,
    # the later one is on top; no rule can tell that from a wire drawing, so the sheet states it.
    pos = {i: n for n, i in enumerate(S.order)}
    firsts = sorted(range(len(regions)), key=lambda k: min(pos[m] for m in regions[k]['members']))
    for rank, k in enumerate(firsts, 1):
        regions[k]['drawn'] = rank
    for t in out_texts:
        t['drawn_after'] = sum(1 for r in regions if min(pos[m] for m in r['members']) < pos[t['id']])
    return regions, out_texts


DEFAULT_BACKING = {'steel': (297, 1), 'stone': (297, 1), 'stone2': (297, 1), 'bevel-dark': (297, 1),
                   'bevel-mid': (897, 1), 'bevel-light': (1040, 1), 'bevel-beige': (1545, 1)}


def backing_words(r):
    """'' when the box has its family's usual backing (INTERFACES.md v2, 2.x); otherwise what it has."""
    fi = r.get('fill_info')
    dflt = DEFAULT_BACKING.get(r['family'])
    if fi is None:
        return ' (no backing of its own)' if dflt else ''
    if dflt and (fi['sprite'], fi['inset']) == dflt and not fi['transparency'] and not fi['flip_h'] and not fi['flip_v']:
        return ''
    s = f' with a {BACKINGS[fi["sprite"]]} filling it' + (' (inset 1px)' if fi['inset'] == 1 else ' (to its edges)')
    if fi['transparency']:
        s += f' (translucent: transparency {fi["transparency"]})'
    if fi['flip_h'] or fi['flip_v']:
        s += ' (mirrored ' + ' and '.join(x for x, on in (('left-right', fi['flip_h']), ('top-bottom', fi['flip_v'])) if on) + ')'
    return s


def words(r):
    f = r['family']
    w, h = r['rect'][2], r['rect'][3]
    if f in ('steel', 'stone', 'stone2'):
        kind = 'steel-border' if f == 'steel' else 'stone-border'
        if r.get('divider') is not None:
            s = f'titled {kind} panel'
            if r.get('close') is not None:
                s += ' with close button'
        else:
            s = f'{kind} inner box'
        return s + backing_words(r)
    if f in FAMILY_WORDS:
        s = FAMILY_WORDS[f] + backing_words(r)
        s = f'button ({s})' if r.get('button') else s
        return s
    if f == 'thin':
        s = 'thin-border box'
        if r.get('divider') is not None:
            s += ' with a header band'
            if r.get('band'):
                b = r['band']
                s += f' (its line {b[2]}x{b[3]} at ({b[0]},{b[1]}) of the box)'
        return s + backing_words(r)
    if f == 'sprite-built':
        return 'button (box built of sprite pieces, not one of the frame families)' if r.get('button') \
            else 'box built of sprite pieces (not one of the frame families)'
    if f == 'close':
        return 'close button'
    if f == 'well':
        s = r['well']
        if r.get('translucent'):
            s += f' (translucent: transparency {r.get("transparency")})'
        if r.get('flip_h') or r.get('flip_v'):
            s += ' (mirrored ' + ' and '.join(x for x, on in (('left-right', r.get('flip_h')), ('top-bottom', r.get('flip_v'))) if on) + ')'
        return s
    if f == 'divider':
        return 'divider strip'
    if f == 'icon':
        return f'icon {r["icon"]}'
    cn = [PAL.name(c) for c in r.get('colours', [])]
    tr = r.get('transparency') or 0
    trs = f', transparency {tr}' if tr else ''
    if f == 'two-tone':
        return f'inner box drawn as a two-tone outline ({cn[0]} outside, {cn[1]} inside{trs})'
    if f == 'outline':
        return f'outline box (1px, {cn[0]}{trs})'
    if f == 'highlight':
        return f'translucent highlight ({cn[0]}{trs})'
    if f == 'filled':
        return f'filled box ({cn[0]})'
    if f == 'bar':
        if r.get('parts'):
            p = r['parts'][-1]
            over = ' over '.join(reversed(cn[1:])) if len(cn) > 1 else ''
            return f'bar ({cn[0]}), with a filled part ({over}) {p[2]}x{p[3]} from its left end'
        if len(cn) > 1:
            return f'bar ({cn[0]}, under {" then ".join(cn[1:])} over its full width)'
        return f'bar ({cn[0]})'
    if f == 'line':
        return f'rule line ({cn[0]}, {r.get("width", 1)}px)'
    if f == 'model':
        return 'model (not scored)'
    if f == 'grid':
        return f'item grid {r["grid"][0]}x{r["grid"][1]}'
    if f == 'item':
        return 'item slot'
    return f
