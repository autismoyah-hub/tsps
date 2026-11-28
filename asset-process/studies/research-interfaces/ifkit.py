"""ifkit.py: the RECIPES of INTERFACES.md v2 as code (builders that return static components).

Every number here is a recipe of INTERFACES.md v2, measured over the drawn 239 groups (recipes/census_*.py) and
verified by drawing it with iface.py against the real instances (recipes/out/frames-summary.txt). Nothing here reads
a target group, a state file or a decode: an author may use it in a RULES run.

Usage:
    K = Kit()                                   # component list for one group (file numbers 0, 1, 2, ...)
    root = K.layer(None, 0, 0, 380, 133, centred=True)
    K.steel_panel(root, 380, 133, 'Title')      # INTERFACES.md 2.1
    box = K.layer(root, 10, 40, 360, 83)
    K.thin_box(box, 360, 83, band_line_y=27)    # 2.5
    K.text(box, ...); K.save('x.json')

Positions are relative to the parent component (mode 0). Pieces are added in the recipe's DRAW ORDER.
"""
import json
from pathlib import Path

PALETTE = None


def pal(name):
    global PALETTE
    if PALETTE is None:
        import ifpalette
        PALETTE = ifpalette.PALETTE
    return PALETTE[name]


FONTS = {'p11': 494, 'p12': 495, 'b12': 496, 'q8': 497}
BACKING, LIGHT, DARK, BEIGE = 297, 1040, 897, 1545
WELLS = {'backing': BACKING, 'light well': LIGHT, 'dark well': DARK, 'beige well': BEIGE}

# 9-slice families with 9px (or 6px) corners: (corners TL TR BL BR, edges L T R B [, middle], corner size, own backing)
BEVELS = {
    'dark bevel box': dict(c=(913, 914, 915, 916), e=(917, 918, 919, 920), k=9, back=BACKING, inset=1),
    'medium bevel box': dict(c=(921, 922, 923, 924), e=(925, 926, 927, 928), k=9, back=DARK, inset=1),
    'light bevel box': dict(c=(929, 930, 931, 932), e=(933, 934, 935, 936), k=9, back=LIGHT, inset=1),
    'beige bevel box': dict(c=(1554, 1555, 1556, 1557), e=(1558, 1559, 1560, 1561), k=9, back=BEIGE, inset=1),
}
TILES = {
    'grey tile box': dict(nw=1141, n=1142, ne=1143, w=1144, c=1145, e=1146, sw=1147, s=1148, se=1149),
    'red tile box': dict(nw=1150, n=1151, ne=1152, w=1153, c=1154, e=1155, sw=1156, s=1157, se=1158),
}


class Kit:
    def __init__(self, group=30900):
        self.group = group
        self.comps = []

    # ------------------------------------------------------------ primitives
    def add(self, kind, parent, x, y, w, h, **kw):
        c = dict(file=len(self.comps), kind=kind, parent=parent, raw_position=[x, y], raw_size=[w, h])
        c.update(kw)
        self.comps.append(c)
        return c['file']

    def layer(self, parent, x, y, w, h, centred=False, fill_parent=False):
        kw = {}
        if centred:
            kw['position_mode'] = [1, 1]
        if fill_parent:
            kw['size_mode'] = [1, 1]
        return self.add(0, parent, x, y, w, h, **kw)

    def sprite(self, parent, sid, x, y, w, h, tiled=True, transparency=0, flip_h=False, flip_v=False):
        return self.add(5, parent, x, y, w, h, sprites=[sid, -1], sprite_tiling=bool(tiled), transparency=transparency,
                        flip_h=bool(flip_h), flip_v=bool(flip_v))

    def rect(self, parent, x, y, w, h, colour, filled=True, transparency=0):
        return self.add(3, parent, x, y, w, h, colors=[colour, 0, 0, 0], filled=bool(filled), transparency=transparency)

    def line(self, parent, x, y, w, h, colour, width=1):
        return self.add(9, parent, x, y, w, h, colors=[colour, 0, 0, 0], line_width=width)

    def text(self, parent, x, y, w, h, s, font, colour, shadow=None, align=(1, 1), line_height=0):
        if shadow is None:
            shadow = shadow_for(colour)
        return self.add(4, parent, x, y, w, h, text=s, font=FONTS.get(font, font), colors=[colour, 0, 0, 0],
                        text_shadow=bool(shadow), text_alignment=list(align), line_height=line_height)

    # ------------------------------------------------------------ 2.1 / 2.2 steel
    def steel_panel(self, parent, W, H, title=None, close=True, backing=True, title_colour=0xff981f, title_font='b12'):
        """2.1: script 227's pieces, in its draw order (holds pixel-exact for 245 boxes in 220 groups). Returns the
        title's file number (or None)."""
        tf = None
        if backing:
            self.sprite(parent, BACKING, 1, 1, W - 2, H - 2)
        if title is not None:
            tf = self.text(parent, 6, 6, W - 12, 24, title, title_font, title_colour, True, (1, 1))
        self.sprite(parent, 310, 0, 0, 25, 30)
        self.sprite(parent, 311, W - 25, 0, 25, 30)
        self.sprite(parent, 312, 0, H - 30, 25, 30)
        self.sprite(parent, 313, W - 25, H - 30, 25, 30)
        self.sprite(parent, 172, -15, 30, 36, H - 60)
        self.sprite(parent, 315, W - 21, 30, 36, H - 60)
        self.sprite(parent, 314, 25, -15, W - 50, 36)
        self.sprite(parent, 173, 25, H - 21, W - 50, 36)
        if title is not None:
            self.sprite(parent, 2546, 5, 14, W - 10, 26)
        if close:
            self.sprite(parent, 535, W - 29, 6, 26, 23)
        return tf

    def steel_box(self, parent, W, H, backing=False):
        """2.2: the steel frame without title, divider or close."""
        self.steel_panel(parent, W, H, None, close=False, backing=backing)

    # ------------------------------------------------------------ 2.3 stone
    def stone_panel(self, parent, W, H, title=None, close=True, backing=True):
        """2.3 (script 703 form; pixel-exact for 29 boxes in 26 groups)."""
        if backing:
            self.sprite(parent, BACKING, 1, 1, W - 2, H - 2)
        for s, x, y in ((824, 0, 0), (825, W - 32, 0), (826, 0, H - 32), (827, W - 32, H - 32)):
            self.sprite(parent, s, x, y, 32, 32)
        self.sprite(parent, 820, 32, -13, W - 64, 32)
        self.sprite(parent, 822, 32, H - 20, W - 64, 32)
        self.sprite(parent, 821, -13, 32, 32, H - 64)
        self.sprite(parent, 823, W - 20, 32, 32, H - 64)
        if title is not None:
            self.sprite(parent, 829, 0, 17, 32, 32)
            self.sprite(parent, 830, W - 32, 17, 32, 32)
            self.sprite(parent, 828, 11, 15, W - 22, 32)
            self.text(parent, 6, 6, W - 12, 23, title, 'b12', 0xff981f, True, (1, 1))
        if close:
            self.sprite(parent, 831, W - 26, 10, 16, 16)

    # ------------------------------------------------------------ 2.5 thin
    def thin_box(self, parent, W, H, band_line_y=None, backing=None, backing_transparency=0):
        """2.5 (script 991 form; pixel-exact for 92 boxes in 13 groups). Draw the box's CONTENTS (wells, texts) BEFORE
        calling this: the frame pieces are drawn last. band_line_y: the top row of the header band's visible 3px
        line, relative to the box (the sheet gives it)."""
        if backing is not None:
            self.sprite(parent, WELLS.get(backing, backing), 0, 0, W, H, transparency=backing_transparency)
        self.sprite(parent, 987, 16, -13, W - 32, 32)
        self.sprite(parent, 989, 16, H - 20, W - 32, 32)
        self.sprite(parent, 988, -14, 16, 32, H - 32)
        self.sprite(parent, 990, W - 20, 16, 32, H - 32)
        self.sprite(parent, 991, 0, 0, 32, 32)
        self.sprite(parent, 992, W - 32, 0, 32, 32)
        self.sprite(parent, 993, 0, H - 32, 32, 32)
        self.sprite(parent, 994, W - 32, H - 32, 32, 32)
        if band_line_y is not None:
            b = band_line_y - 17
            self.sprite(parent, 995, 0, b, W, 32)
            self.sprite(parent, 996, 0, b + 2, 32, 32)
            self.sprite(parent, 997, W - 32, b + 2, 32, 32)

    # ------------------------------------------------------------ 2.4 / 2.6 bevels, 2.7 tiles
    def bevel_box(self, parent, family, W, H, backing=True):
        """2.4/2.6: 9px corners and 9px edge canvases (pixel-exact: dark 239 boxes in 67 groups, medium 52 in 34,
        light 119 in 30, beige 36 in 9). Own backing at (1,1,W-2,H-2), drawn first."""
        f = BEVELS[family]
        k = f['k']
        if backing:
            b = f['back'] if backing is True else WELLS.get(backing, backing)
            self.sprite(parent, b, 1, 1, W - 2, H - 2)
        tl, tr, bl, br = f['c']
        el, et, er, eb = f['e']
        self.sprite(parent, tl, 0, 0, k, k)
        self.sprite(parent, tr, W - k, 0, k, k)
        self.sprite(parent, bl, 0, H - k, k, k)
        self.sprite(parent, br, W - k, H - k, k, k)
        self.sprite(parent, el, 0, k, k, H - 2 * k)
        self.sprite(parent, et, k, 0, W - 2 * k, k)
        self.sprite(parent, er, W - k, k, k, H - 2 * k)
        self.sprite(parent, eb, k, H - k, W - 2 * k, k)

    def tile_box(self, parent, family, W, H):
        """2.7: 6px corners NOT tiled, edges and middle tiled (pixel-exact for 72 boxes in 32 groups, red 9 in 8)."""
        t = TILES[family]
        self.sprite(parent, t['nw'], 0, 0, 6, 6, tiled=False)
        self.sprite(parent, t['n'], 6, 0, W - 12, 6)
        self.sprite(parent, t['ne'], W - 6, 0, 6, 6, tiled=False)
        self.sprite(parent, t['w'], 0, 6, 6, H - 12)
        self.sprite(parent, t['c'], 6, 6, W - 12, H - 12)
        self.sprite(parent, t['e'], W - 6, 6, 6, H - 12)
        self.sprite(parent, t['sw'], 0, H - 6, 6, 6, tiled=False)
        self.sprite(parent, t['s'], 6, H - 6, W - 12, 6)
        self.sprite(parent, t['se'], W - 6, H - 6, 6, 6, tiled=False)

    # ------------------------------------------------------------ 3 wells, 4 rectangles
    def well(self, parent, kind, x, y, w, h, transparency=0, flip_h=False, flip_v=False):
        """3: a tiled backing; its tiles start at its own top-left."""
        return self.sprite(parent, WELLS.get(kind, kind), x, y, w, h, True, transparency, flip_h, flip_v)

    def two_tone(self, parent, x, y, w, h, outer=0x0e0e0c, inner=0x474745, transparency=0):
        """4: an outline and a second one 1px inside it (script 714: 0e0e0c outside, 474745 inside)."""
        self.rect(parent, x, y, w, h, outer, False, transparency)
        self.rect(parent, x + 1, y + 1, w - 2, h - 2, inner, False, transparency)

    # ------------------------------------------------------------ 7 HPBAR_HUD (242 runtime form)
    def hpbar_hud(self, parent, x, y, name, current, maximum, W=150, H=42):
        """7.1: the boss health bar as TSPS's 242 scripts lay it out (holds for 303 in the live state)."""
        box = self.layer(parent, x, y, W, H)
        self.rect(box, 1, 1, W - 2, H - 2, 0x191511, False)
        self.well(box, 'dark well', 2, 2, W - 4, H - 4)
        nl = self.layer(box, 0, 0, W, 19)
        self.text(nl, 8, 2, W - 16, 19, name, 'p11', 0xff981f, True, (1, 1))
        bl = self.layer(box, 3, 18, W - 6, 21)
        self.rect(bl, 1, 1, W - 8, 19, 0x191511, False)
        inner = self.layer(bl, 2, 2, W - 10, 17)
        fill = (W - 10) * current // maximum
        self.rect(inner, 0, -2, W - 10, 21, 0xc80000)
        self.rect(inner, 0, -2, fill, 21, 0x009000)
        self.rect(inner, 0, -2, fill, 21, 0x00c800)
        self.text(bl, 0, 1, W - 6, 21, f'{current} / {maximum}', 'p11', 0xeaeaea, True, (1, 1))
        return box

    # ------------------------------------------------------------ out
    def save(self, path, **top):
        d = dict(components=self.comps)
        d.update(top)
        Path(path).write_text(json.dumps(d, indent=1))
        return path


DARK_TEXT = {0x000000, 0x403020, 0x46320a, 0x800000, 0x0000ff, 0x4c3d26, 0x282315, 0x37242b, 0x2f2a25, 0x505050}


def shadow_for(colour):
    """3.3: light text has the 1px black shadow, dark text (on parchment and beige) has none."""
    c = colour & 0xffffff
    if c in DARK_TEXT:
        return False
    r, g, b = c >> 16, (c >> 8) & 255, c & 255
    return (r * 299 + g * 587 + b * 114) / 1000 > 60


def inv_init_pitch(W, H, columns, rows):
    """6: 36x32 slots, first flush top-left, last flush bottom-right."""
    px = (W - 36) // (columns - 1) if columns > 1 else 0
    py = (H - 32) // (rows - 1) if rows > 1 else 0
    return px, py
