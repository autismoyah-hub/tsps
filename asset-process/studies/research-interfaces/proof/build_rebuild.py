"""build_rebuild.py: the proof rebuild of PET_INSURANCE (148), written from INTERFACES.md's rules plus what
measure_sheet.py and fit_backing.py measured on the picture (proof/sheet-148-as-tsps-draws-it.png).
The real panel's component list (cache or state) was not opened before the rebuild matched; see NEXT.md, section 3.

Writes proof/rebuild-148.json (a component list iface.py draws, group 30900) and proof/rebuild-148-choices.json."""
import sys, json
from pathlib import Path
import numpy as np
from PIL import Image

HERE = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(HERE))
import ifcache as C, iface

GROUP = 30900
W, H = 400, 300
comps = []


def add(kind, parent, x, y, w, h, **kw):
    f = len(comps)
    c = dict(id=(GROUP << 16) | f, group=GROUP, file=f, if3=True, kind=kind, button=0, content=0,
             raw_position=[x, y], raw_size=[w, h], size_mode=[0, 0], position_mode=[0, 0],
             parent=None if parent is None else (GROUP << 16) | parent, hidden=False, mouse_over=-1,
             scroll_size=[0, 0], no_click_through=False, colors=[0, 0, 0, 0], filled=False, transparency=0, fill_mode=0,
             line_width=1, line_reversed=False, sprites=[-1, -1], sprite_angle=0, sprite_tiling=False, outline=0,
             sprite_shadow=0, flip_v=False, flip_h=False, models=[-1, -1], sequences=[-1, -1], model_offset=[0, 0],
             model_angles=[0, 0, 0], model_zoom=100, model_ortho=False, model_extra=0, model_scale=[0, 0], font=-1,
             text='', alternate_text='', text_alignment=[0, 0], line_height=0, text_shadow=False, flags=0,
             data_text='', actions=[], drag_zone=0, drag_threshold=0, scrollbar=False, spell_action='', spell_name='',
             button_text='Ok', listeners={}, triggers={}, cs1_comparisons=[], cs1=[])
    c.update(kw)
    comps.append(c)
    return f


def sprite(parent, sid, x, y, w, h, tiled=True, **kw):
    return add(5, parent, x, y, w, h, sprites=[sid, -1], sprite_tiling=tiled, **kw)


def steel_box(parent, x, y, w, h, background=None):
    """INTERFACES.md 2.2: steel corners and edges; 2.1's background when asked."""
    if background is not None:
        sprite(parent, background[0], x + 1, y + 1, w - 2, h - 2)
    sprite(parent, 310, x, y, 25, 30); sprite(parent, 311, x + w - 25, y, 25, 30)
    sprite(parent, 312, x, y + h - 30, 25, 30); sprite(parent, 313, x + w - 25, y + h - 30, 25, 30)
    sprite(parent, 172, x - 15, y + 30, 36, h - 60); sprite(parent, 315, x + w - 21, y + 30, 36, h - 60)
    sprite(parent, 314, x + 25, y - 15, w - 50, 36); sprite(parent, 173, x + 25, y + h - 21, w - 50, 36)


def text_top(font, s, box_y, height):
    """Row of the first inked pixel of s drawn top-aligned at box_y (to place text where it was measured)."""
    cache = C.Cache(C.CACHE_242)
    r = iface.Raster(200, 120, 0xff000000)
    f = C.font(cache, font)
    iface.draw_paragraph(r, f, s, [0, box_y, 200, height], dict(color=0xffffff, shadow=None, alpha=256), [0, 0], 0, [])
    ink = np.nonzero(((r.px & 0xffffff) != 0).any(1))[0]
    xs = np.nonzero(((r.px & 0xffffff) != 0).any(0))[0]
    return int(ink.min()), int(xs.max() - xs.min() + 1)


def main():
    root = add(0, None, 0, 0, W, H, position_mode=[1, 1])
    # 2.1 steel border with title and close, title from the sheet
    steel_box(root, 0, 0, W, H, background=(297,))
    sprite(root, 2546, 5, 14, W - 10, 26)
    add(4, root, 6, 6, W - 12, 24, font=496, text='Pet Insurance', colors=[0xff981f, 0, 0, 0], text_shadow=True,
        text_alignment=[1, 1])
    sprite(root, 535, W - 29, 6, 26, 23, tiled=True)
    # measured boxes (corner sprites found at these places on the sheet): right box D first, then A, B, C on top
    boxes = dict(D=(145, 30, 255, 270), A=(0, 30, 150, 104), B=(0, 130, 150, 106), C=(0, 232, 150, 68))
    choices = dict(boxes=boxes)
    for name in ('D', 'A', 'B', 'C'):
        x, y, w, h = boxes[name]
        if name in ('A', 'C'):
            # light wells: measured as TRADEBACKING_LIGHT 1040, flipped left-right, tiled from the box's own corner
            sprite(root, 1040, x, y, w, h, flip_h=True)
        steel_box(root, x, y, w, h)
    choices['wells'] = 'sprite 1040 flipped horizontally, tiled from the box corner (fit: tile phase x 0 mod 88, y 30 mod 60)'
    # text: font by measured width, row by measured top (sheet: 'Reclaim Tokens' ink x 27-122, y 144-153, ff981f;
    # '0' ink x 71-77, y 162-171, ffffff)
    best = None
    for fid in (496, 495, 494):
        top, width = text_top(fid, 'Reclaim Tokens', 20, 30)
        if best is None or abs(width - (122 - 27)) < best[1]:
            best = (fid, abs(width - (122 - 27)), top - 20)
    fid, _, ink_off = best
    choices['title_font'] = fid
    add(4, root, 0, 144 - ink_off, 150, 15, font=fid, text='Reclaim Tokens', colors=[0xff981f, 0, 0, 0],
        text_shadow=True, text_alignment=[1, 0])
    best0 = None
    for f0 in (496, 495, 494):
        top0, w0 = text_top(f0, '0', 20, 30)
        if best0 is None or abs(w0 - (77 - 71 + 1)) < best0[1]:
            best0 = (f0, abs(w0 - 7), top0 - 20)
    f0, _, off0 = best0
    choices['number_font'] = f0
    add(4, root, 0, 162 - off0, 150, 15, font=f0, text='0', colors=[0xffffff, 0, 0, 0], text_shadow=True,
        text_alignment=[1, 0])
    (HERE / 'proof' / 'rebuild-148.json').write_text(json.dumps(dict(components=comps), indent=0))
    (HERE / 'proof' / 'rebuild-148-choices.json').write_text(json.dumps(choices, indent=1))
    print(len(comps), 'components;', choices)


if __name__ == '__main__':
    main()
