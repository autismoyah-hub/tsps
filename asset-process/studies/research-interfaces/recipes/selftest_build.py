"""selftest_build.py SHEET_DIR OUT.json: rebuild a panel from its RULES sheet using ONLY the recipes of INTERFACES.md
v2 (ifkit.py) and the sheet (sheet.json). No decode, state, control or teacher file is read.

The only measurement is of the builder's OWN drawing (allowed in a rules run): a text's font is the one of the
four main fonts whose letters, drawn alone, have exactly the sheet's letters size; its box is then shifted until
its letters sit exactly on the sheet's letters box.

Region words -> recipe (INTERFACES.md v2 section):
  titled steel-border panel ............ 2.1 ifkit.steel_panel (title: the 'title' string)
  steel-border inner box ............... 2.2 ifkit.steel_box (backing unless "no backing of its own")
  titled stone-border panel ............ 2.3 ifkit.stone_panel
  thin-border box [with a header band (its line .. at (x,y) of the box)] ... 2.5 ifkit.thin_box, contents first
  dark/medium/light/beige bevel box .... 2.4/2.6 ifkit.bevel_box; button (...) the same with its label inside
  grey/red tile box .................... 2.7 ifkit.tile_box
  backing / light / dark / beige well .. 3 ifkit.well (transparency, mirroring as the sheet says)
  outline box, two-tone outline, filled box, translucent highlight, rule line ... 4 (palette colours)
  bar (c), with a filled part (c2) ..... 7 (base at the bar, part from its left end)
Texts (section 5): font by letters size; colour by palette name; shadow by the light/dark rule; alignment centred
both ways in its recipe box (title box, button box, table cell) and otherwise centred on its letters; line pitch
and line alignment from the sheet; emphasised parts as <col> spans of their palette colours.
"""
import sys, json, re
from pathlib import Path
import numpy as np

HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))
import ifkit as KIT
import ifpalette as PAL
import ifcache as C
import iface
import ifref as R   # used ONLY for rebuild_scene / normalize / ink: drawing the builder's own JSON

iface.TSPS_BLEND = True
FONT_ORDER = ['p12', 'p11', 'b12', 'q8']
_cache = None


def cache():
    global _cache
    if _cache is None:
        _cache = C.Cache(C.CACHE_242)
    return _cache


def inside(a, b, tol=0):
    return a[0] >= b[0] - tol and a[1] >= b[1] - tol and a[0] + a[2] <= b[0] + b[2] + tol and a[1] + a[3] <= b[1] + b[3] + tol


def markup(lines, emph, base):
    """The text with <br> between the sheet's lines and <col> spans for emphasised parts."""
    spans = []
    for e in emph:
        m = re.match(r'^(.*) \(([^()]*)\)$', e)
        if m:
            spans.append((m.group(1), PAL.PALETTE.get(m.group(2))))
    # per character colour over the lines joined by '\n'; a part may run across a line break
    full = '\n'.join(lines)
    cols = [None] * len(full)
    pos = 0
    for part, col in spans:
        if col is None:
            continue
        pat = r'\s+'.join(re.escape(w) for w in part.split())
        m = re.compile(pat).search(full, pos) or re.compile(pat).search(full)
        if m:
            for i in range(m.start(), m.end()):
                cols[i] = col
            pos = m.end()
    out, cur = '', None
    for ch, c in zip(full, cols):
        if ch == '\n':
            if cur is not None:
                out += '</col>'
            out += '<br>'
            cur = None   # <br> ends a colour span (the client's markup); reopen it on the next line
            continue
        if c != cur:
            if cur is not None:
                out += '</col>'
            if c is not None:
                out += f'<col={c:06x}>'
            cur = c
        out += ch
    if cur is not None:
        out += '</col>'
    return out


def own_ink(comps, pane, file_no):
    """The letters box (x, y, w, h) in the pane of component file_no of the builder's own list."""
    n = R.normalize(json.loads(json.dumps(comps)))
    S = R.rebuild_scene(n, pane)
    cid = next(c['id'] for c in n if c['file'] == file_no)
    return S.ink(cid)[1]


class Builder:
    def __init__(self, sheet):
        self.sh = sheet
        self.k = KIT.Kit(30900)
        self.pane = 'hud' if sheet['opens_in'] == 'HUD overlay' else 'modal'
        self.W, self.H = sheet['panel_size']
        self.org = sheet['panel_in_pane']
        self.done_texts = set()

    def font_for(self, t, text, colour, pitch, align):
        """The main font whose drawing of the text has exactly the sheet's letters size."""
        L = t['letters']
        best = None
        for f in FONT_ORDER:
            fnt = C.font(cache(), KIT.FONTS[f])
            lh = 0 if pitch in (None, fnt.metrics.ascent) else pitch
            k = KIT.Kit()
            root = k.layer(None, 0, 0, 1230, 708)
            k.text(root, 100, 100, 600, 300, text, f, colour, None, (align, 1), lh)
            n = R.normalize(k.comps)
            S = R.rebuild_scene(n, 'hud')
            b = S.ink(n[1]['id'])[1]
            if b and b[2] == L[2] and b[3] == L[3]:
                return f, lh
            if b and best is None and abs(b[2] - L[2]) <= 1 and abs(b[3] - L[3]) <= 1:
                best = (f, lh)
        return best or ('p12', 0 if pitch in (None, 12) else pitch)

    def place_text(self, parent, pabs, t, box, align=(1, 1)):
        """Add string t with its recipe box (relative to parent), then shift it so its letters land on the sheet's."""
        colour = PAL.PALETTE.get(t['colour'], 0xff981f)
        text = markup(t['lines'], t['emphasised'], colour)
        la = {'left': 0, 'centre': 1, 'right': 2}.get(t.get('lines_aligned') or '', align[0])
        align = (la, align[1])
        f, lh = self.font_for(t, text, colour, t.get('line_pitch'), align[0])
        x, y, w, h = box
        fn = self.k.text(parent, x, y, w, h, text, f, colour, None, align, lh)
        return self.adjust(fn, t)

    def adjust(self, fn, t):
        """Shift text component fn until its letters (measured on the builder's own drawing) sit on the sheet's."""
        want = [t['letters'][0] + self.org[0], t['letters'][1] + self.org[1]]
        for _ in range(3):
            got = own_ink(self.k.comps, self.pane, fn)
            if got is None:
                break
            dx, dy = want[0] - got[0], want[1] - got[1]
            if not dx and not dy:
                break
            c = self.k.comps[fn]
            c['raw_position'] = [c['raw_position'][0] + dx, c['raw_position'][1] + dy]
        self.done_texts.add(t['n'])
        return fn

    def build(self):
        sh, k = self.sh, self.k
        W, H = self.W, self.H
        regions = [dict(r) for r in sh['regions']]
        texts = sh['strings']
        if self.pane == 'hud':
            root = k.layer(None, 0, 0, 0, 0, fill_parent=True)
            base = k.layer(root, self.org[0], self.org[1], W, H)
        else:
            root = k.layer(None, 0, 0, W, H, centred=True)
            base = root
        babs = [0, 0]
        # the panel frame first (it holds the title), then every other region in sheet order
        frame = next((r for r in regions if r['role'].startswith('titled ') and r['rect'][:2] == [0, 0]), None)
        if frame:
            title = next((t for t in texts if t['role'] == 'title'), None)
            close = 'close' in frame['role']
            if 'steel' in frame['role']:
                # 2.1: the title in (6, 6, W-12, 24), centred both ways, drawn right after the backing
                if title:
                    colour = PAL.PALETTE.get(title['colour'], 0xff981f)
                    text = markup(title['lines'], title['emphasised'], colour)
                    f, _ = self.font_for(title, text, colour, None, 1)
                    tf = k.steel_panel(base, W, H, text, close=close, title_colour=colour, title_font=f)
                    self.adjust(tf, title)
                else:
                    k.steel_panel(base, W, H, None, close=close)
            else:
                k.stone_panel(base, W, H, None, close=close)
                if title:
                    self.place_text(base, babs, title, [6, 6, W - 12, 23])
        self.emit_all(base, regions, texts, frame)
        return k

    # ------------------------------------------------------------ regions as a tree, in the sheet's draw order
    CONTAINERS = ('thin-border box', 'steel-border inner box', 'button (')

    def is_container(self, r):
        return r['role'].startswith(self.CONTAINERS) or any(f in r['role'] for f in list(KIT.BEVELS) + list(KIT.TILES))

    def emit_all(self, base, regions, texts, frame):
        others = [r for r in regions if r is not frame]
        cont = [r for r in others if self.is_container(r)]

        def holder(rect, me=None):
            cands = [c for c in cont if c is not me and inside(rect, c['rect']) and c['rect'] != rect]
            return min(cands, key=lambda c: c['rect'][2] * c['rect'][3]) if cands else None
        self.kids = {id(c): [] for c in cont}
        self.tkids = {id(c): [] for c in cont}
        top, ttop = [], []
        for r in others:
            h = holder(r['rect'], r)
            (self.kids[id(h)] if h else top).append(r)
        for t in texts:
            if t['n'] in self.done_texts:
                continue
            h = holder(t['letters'])
            (self.tkids[id(h)] if h else ttop).append(t)
        for r in sorted(top, key=lambda r: r['drawn']):
            self.emit(r, base, [0, 0])
        # remaining texts: centred on their letters, in the base layer (or the HUD box)
        for t in ttop:
            lx, ly, lw, lh = t['letters']
            self.place_text(base, [0, 0], t, [lx - 4, ly - 4, lw + 8, lh + 8])

    def emit(self, r, par, pab):
        k = self.k
        role = r['role']
        x, y, w, h = r['rect']
        rx, ry = x - pab[0], y - pab[1]
        if self.is_container(r):
            L = k.layer(par, rx, ry, w, h)
            kids = sorted(self.kids[id(r)], key=lambda q: q['drawn'])
            texts = self.tkids[id(r)]
            if role.startswith('thin-border box'):
                # 2.5: contents (wells, rectangles, texts) first, the frame pieces last
                for q in kids:
                    self.emit(q, L, [x, y])
                self.table_texts(L, r, texts)
                bm = re.search(r'its line (\d+)x(\d+) at \((\d+),(\d+)\)', role)
                bk = re.search(r'with a (backing|light well|dark well|beige well) filling it', role)
                tr = re.search(r'filling it[^;]*?transparency (\d+)', role)
                k.thin_box(L, w, h, band_line_y=int(bm.group(4)) if bm else None,
                           backing=bk.group(1) if bk else None, backing_transparency=int(tr.group(1)) if tr else 0)
                return
            if role.startswith('steel-border inner box'):
                # 2.2: the box's own backing (if any) first, then its contents, then its frame pieces
                self.own_backing(L, r, w, h, default=('backing', 1))
                for q in kids:
                    self.emit(q, L, [x, y])
                for t in texts:
                    lx, ly, lw, lh = t['letters']
                    self.place_text(L, [x, y], t, [lx - x - 4, ly - y - 4, lw + 8, lh + 8])
                k.steel_box(L, w, h, backing=False)
                return
            fam = next((f for f in list(KIT.BEVELS) + list(KIT.TILES) if f in role), None)
            # 2.4/2.6/2.7: a bevel or tile box, then what is inside it; a button's label fills the box (5.4)
            if fam in KIT.TILES:
                k.tile_box(L, fam, w, h)
            else:
                dflt = {'dark bevel box': 'backing', 'medium bevel box': 'dark well', 'light bevel box': 'light well',
                        'beige bevel box': 'beige well'}[fam]
                self.own_backing(L, r, w, h, default=(dflt, 1))
                k.bevel_box(L, fam, w, h, backing=False)
            for q in kids:
                self.emit(q, L, [x, y])
            for t in texts:
                box = [0, 0, w, h] if role.startswith('button') else None
                if box is None:
                    lx, ly, lw, lh = t['letters']
                    box = [lx - x - 4, ly - y - 4, lw + 8, lh + 8]
                self.place_text(L, [x, y], t, box)
            return
        self.leaf(r, par, rx, ry, w, h)

    def own_backing(self, L, r, w, h, default):
        role = r['role']
        if 'no backing of its own' in role:
            return
        bk = re.search(r'with a (backing|light well|dark well|beige well) filling it( \(inset 1px\)| \(to its edges\))?', role)
        kind, d = default
        if bk:
            kind = bk.group(1)
            d = 0 if bk.group(2) and 'edges' in bk.group(2) else 1
        tr = re.search(r'filling it[^;]*?transparency (\d+)', role)
        self.k.well(L, kind, d, d, w - 2 * d, h - 2 * d, int(tr.group(1)) if tr else 0,
                    'left-right' in role, 'top-bottom' in role)

    def table_texts(self, L, r, inner):
        """2.5 / 5.5: in a thin box with a header band, headings are centred in their column above the band line,
        cells below it; columns are equal, as wide as the distance between neighbouring heading centres."""
        x, y, w, h = r['rect']
        bm = re.search(r'its line (\d+)x(\d+) at \((\d+),(\d+)\)', r['role'])
        line_y = int(bm.group(4)) if bm else None
        heads = [t for t in inner if line_y is not None and t['letters'][1] < y + line_y]
        centres = sorted(t['letters'][0] + t['letters'][2] / 2 - x for t in heads)
        pitch = min((b - a for a, b in zip(centres, centres[1:])), default=None)
        pitch = int(round(pitch)) if pitch else None
        for t in inner:
            cx = t['letters'][0] + t['letters'][2] / 2 - x
            cx0, cw = (int(cx // pitch) * pitch, pitch) if pitch else (0, w)
            if line_y is not None and t in heads:
                cell = [cx0, 0, cw, line_y + 3]
            elif line_y is not None:
                cell = [cx0, line_y + 3, cw, h - line_y - 4]
            else:
                cell = [cx0, 0, cw, h]
            self.place_text(L, [x, y], t, cell)

    def leaf(self, r, par, rx, ry, w, h):
        k = self.k
        role = r['role']
        m = re.match(r'^(backing|light well|dark well|beige well)', role)
        if m:
            tr = re.search(r'transparency (\d+)', role)
            k.well(par, m.group(1), rx, ry, w, h, int(tr.group(1)) if tr else 0, 'left-right' in role,
                   'top-bottom' in role)
            return
        groups = re.findall(r'\(([^()]*)\)', role)
        names = []
        for grp in groups:
            for c in re.split(r',| over | then | under ', grp):
                c = re.sub(r'\b(outside|inside|1px|\d+px|transparency \d+|over its full width)\b', '', c).strip()
                if c in PAL.PALETTE:
                    names.append(c)
        pc = [PAL.PALETTE[n] for n in names]
        tr = re.search(r'transparency (\d+)', role)
        tr = int(tr.group(1)) if tr else 0
        if role.startswith('outline box'):
            k.rect(par, rx, ry, w, h, pc[0], False, tr)
        elif role.startswith('inner box drawn as a two-tone'):
            k.two_tone(par, rx, ry, w, h, pc[0], pc[1], tr)
        elif role.startswith('filled box'):
            k.rect(par, rx, ry, w, h, pc[0], True)
        elif role.startswith('translucent highlight'):
            k.rect(par, rx, ry, w, h, pc[0], True, tr)
        elif role.startswith('bar'):
            k.rect(par, rx, ry, w, h, pc[0], True)
            pm = re.search(r'filled part \(([^()]*)\) (\d+)x(\d+)', role)
            if pm:
                for nm in reversed([s.strip() for s in pm.group(1).split(' over ')]):
                    k.rect(par, rx, ry, int(pm.group(2)), int(pm.group(3)), PAL.PALETTE[nm], True)
            else:
                for nm in pc[1:]:
                    k.rect(par, rx, ry, w, h, nm, True)
        elif role.startswith('rule line'):
            wd = re.search(r'(\d+)px', role)
            k.line(par, rx, ry, w if h <= 1 else 0, h if w <= 1 else 0, pc[0], int(wd.group(1)) if wd else 1)
        # icons, models, items, grids, sprite-built boxes, dividers: not built (undecidable from a rules sheet)


def main():
    sd, out = Path(sys.argv[1]), Path(sys.argv[2])
    sheet = json.loads((sd / 'sheet.json').read_text())
    b = Builder(sheet)
    k = b.build()
    top = {'pane': 'hud'} if b.pane == 'hud' else {}
    k.save(out, **top)
    print(f'{out}: {len(k.comps)} components')


if __name__ == '__main__':
    main()
