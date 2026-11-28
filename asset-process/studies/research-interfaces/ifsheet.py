"""ifsheet.py GROUP OUTDIR --kind rules|picture

Make the SHEET an author gets to rebuild a real panel blind (NEXT.md section 1), and the Teacher's own folder.

The sheet holds NO component list, decode, sprite id, font or colour of the panel. It holds:
  - sheet.md / sheet.json: the panel's size, the pane it opens in and where; every visually distinct REGION as a
    rectangle with its size in pixels and its role in words ("titled steel-border panel", "button (light bevel
    box)", "light well (translucent)", "icon B", "bar, with a filled part 79x17 from its left end", ...); every
    visible STRING in reading order with its role (title, heading, label, value, body, button), its lines, the
    box its letters cover and the parts drawn in another colour than the rest (named as parts, not as colours);
  - wire.png: the wire drawing of the same at 2x (colours there mean kinds of region, not the panel's colours);
  - with --kind picture only: panel-1x.png and panel-3x.png, the panel as the TSPS client drew it.

The regions come from the real panel by a fixed procedure (ifref.classify): drawn components are grouped into
framed boxes by their corner sprites (the frame families of INTERFACES.md section 2), sprite-built boxes (a layer
whose own sprites cover its corners and 90% of it), close buttons, backings and wells (tiled backing sprites),
dividers, icons (any other sprite; equal pictures get the same letter), two-tone and single outlines, translucent
highlights, filled boxes, bars (opaque fills at most 21px tall; a shorter fill on the same row is its filled part),
rule lines, models, items and item grids. A bevel or sprite-built box with a text component inside is a button.
Text roles: inside a button: button; above the divider of a titled frame: title (heading for a header band);
otherwise body if over 40 characters, value if it is numbers, else label. Rectangles are what is VISIBLE (a
frame's outer edge, a sprite's inked box, a text's letters), relative to the panel's top-left.

Teacher-only (sheets-teacher-only/GROUP/, authors must not read it): real-1x.png, real-3x.png, the subtree of the
state (state.jsonl), the control component list (control.json: the real panel in the rebuild form), regions with
their member components (classify.json), meta.json. ifgate.py reads them.
"""
import os as _ap_os, sys as _ap_sys  # find asset_paths.py (see SETUP.md)
_ap_d = _ap_os.path.dirname(_ap_os.path.abspath(globals().get('__file__') or 'x'))
while not _ap_os.path.isfile(_ap_os.path.join(_ap_d, 'asset_paths.py')) and _ap_os.path.dirname(_ap_d) != _ap_d: _ap_d = _ap_os.path.dirname(_ap_d)
_ap_sys.path.append(_ap_d); import asset_paths as _AP  # noqa: E402
import sys, json, argparse
from pathlib import Path
import numpy as np
from PIL import Image, ImageDraw, ImageFont

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import ifref as R

TEACHER = HERE / 'sheets-teacher-only'
SCALE = 2

CLASS_COL = {
    'frame': (90, 200, 255), 'inner': (255, 220, 80), 'button': (100, 230, 120), 'close': (255, 120, 120),
    'well': (230, 150, 60), 'highlight': (150, 200, 255), 'icon': (230, 100, 230), 'bar': (255, 80, 80),
    'model': (160, 160, 160), 'text': (235, 235, 235), 'line': (200, 200, 120), 'grid': (120, 255, 220),
}


def region_class(r):
    f = r['family']
    if f in ('steel', 'stone', 'stone2'):
        return 'frame' if r.get('divider') is not None else 'inner'
    if r.get('button'):
        return 'button'
    if f in ('thin', 'two-tone', 'outline', 'bevel-dark', 'bevel-light', 'bevel-mid', 'bevel-beige', 'tile-grey',
             'tile-red', 'sprite-built', 'filled'):
        return 'inner'
    if f == 'close':
        return 'close'
    if f == 'well':
        return 'well'
    if f == 'highlight':
        return 'highlight'
    if f == 'icon':
        return 'icon'
    if f == 'bar':
        return 'bar'
    if f in ('model', 'item'):
        return 'model'
    if f == 'grid':
        return 'grid'
    return 'line'


def opens_in(S, panel):
    pw, ph = S.pane
    x, y, w, h = panel
    if S.kind == 'hud':
        where = (f'an overlay in the HUD pane (TSPS resizable layout, {pw}x{ph} in the capture); its visible box '
                 f'sits {x} px from the pane\'s left and {y} px from its top (anchored top-left)')
        return 'HUD overlay', where
    cx = abs(x - (pw - w) / 2) <= 1
    cy = abs(y - (ph - h) / 2) <= 1
    place = ('centred both ways' if cx and cy else 'centred horizontally' if cx else 'centred vertically' if cy
             else 'placed absolutely')
    return 'main modal', f'the main modal (fixed mode {pw}x{ph}); drawn at ({x},{y}) of it: {place}'


def font(size):
    for f in (_AP.fix('<FONT_DIR>/consola.ttf'), _AP.fix('<FONT_DIR>/arial.ttf')):
        try:
            return ImageFont.truetype(f, size)
        except OSError:
            pass
    return ImageFont.load_default()


def wire(panel, regions, texts, out):
    x0, y0, W, H = panel
    pad = 24
    legend_w = 620
    lines = []
    for n, r in enumerate(regions, 1):
        x, y, w, h = r['rect']
        lines.append((region_class(r), f'R{n}  {r["words"]}  {w}x{h} at ({x - x0},{y - y0})'))
    for n, t in enumerate(texts, 1):
        x, y, w, h = t['ink']
        s = ' / '.join(t['lines'])
        if len(s) > 52:
            s = s[:49] + '...'
        lines.append(('text', f'T{n}  {t["role"]}: "{s}"  letters {w}x{h} at ({x - x0},{y - y0})'))
    f12, f11 = font(13), font(11)
    img_h = max(H * SCALE + 2 * pad + 30, 40 + 17 * len(lines) + 120)
    img = Image.new('RGB', (W * SCALE + 2 * pad + legend_w, img_h), (24, 24, 28))
    d = ImageDraw.Draw(img)
    d.text((pad, 6), f'panel {W}x{H} (drawn at {SCALE}x; numbers are panel pixels)', fill=(220, 220, 220), font=f12)
    ox, oy = pad, pad + 10

    def R_(x, y, w, h):
        return [ox + (x - x0) * SCALE, oy + (y - y0) * SCALE, ox + (x - x0 + w) * SCALE - 1, oy + (y - y0 + h) * SCALE - 1]
    d.rectangle(R_(x0, y0, W, H), outline=(255, 255, 255), width=1)
    for n, r in enumerate(regions, 1):
        c = CLASS_COL[region_class(r)]
        box = R_(*r['rect'])
        wd = 3 if region_class(r) == 'frame' else 2 if region_class(r) in ('inner', 'button', 'bar') else 1
        d.rectangle(box, outline=c, width=wd)
        if r['family'] == 'bar' and r.get('parts'):
            for p in r['parts']:
                d.rectangle(R_(*p), outline=c, width=1)
                d.line([box[0], box[1], R_(*p)[2], box[3]], fill=c, width=1)
        if r.get('band'):
            b = r['band']
            bx, by_ = r['rect'][:2]
            d.rectangle(R_(bx + b[0], by_ + b[1], b[2], b[3]), outline=c, width=1)
        tag = f'R{n}' if r['family'] != 'icon' else f'R{n}:{r["icon"]}'
        tw = d.textlength(tag, font=f11)
        d.rectangle([box[0] + 1, box[1] + 1, box[0] + tw + 4, box[1] + 13], fill=(24, 24, 28))
        d.text((box[0] + 3, box[1] + 1), tag, fill=c, font=f11)
        if r['family'] not in ('icon',) and r['rect'][2] * SCALE > 80:
            sz = f'{r["rect"][2]}x{r["rect"][3]}'
            d.text((box[2] - d.textlength(sz, font=f11) - 3, box[3] - 13), sz, fill=c, font=f11)
    for n, t in enumerate(texts, 1):
        box = R_(*t['ink'])
        d.rectangle(box, outline=CLASS_COL['text'], width=1)
        s = f'T{n} ' + ' / '.join(t['lines'])
        maxw = box[2] - box[0] - 4
        if d.textlength(s, font=f11) > maxw:
            s = f'T{n}'
        d.text((box[0] + 2, box[1] + 1), s, fill=(200, 200, 200), font=f11)
    lx = W * SCALE + 2 * pad
    d.text((lx, 6), 'Legend (wire colours = kind of region, not the panel\'s colours)', fill=(220, 220, 220), font=f12)
    yy = 28
    for cls, s in [('frame', 'titled frame'), ('inner', 'inner box / framed box'), ('button', 'button'),
                   ('close', 'close button'), ('well', 'backing or well'), ('highlight', 'translucent highlight'),
                   ('icon', 'icon (same letter = same picture)'), ('bar', 'bar and its filled part'),
                   ('model', 'model or item (not scored)'), ('text', 'text: box of its letters')]:
        d.rectangle([lx, yy + 3, lx + 12, yy + 12], outline=CLASS_COL[cls], width=2)
        d.text((lx + 18, yy), s, fill=CLASS_COL[cls], font=f11)
        yy += 14
    yy += 8
    for cls, s in lines:
        d.text((lx, yy), s, fill=CLASS_COL[cls], font=f11)
        yy += 16
    img.save(out)


def make(group, outdir, kind):
    S = R.real_scene(group)
    t = R.ensure_target(group)
    panel = S.panel_rect()
    regions, texts = R.classify(S)
    x0, y0 = panel[:2]
    where_short, where = opens_in(S, panel)
    outdir = Path(outdir); outdir.mkdir(parents=True, exist_ok=True)
    tdir = TEACHER / str(group); tdir.mkdir(parents=True, exist_ok=True)

    def rel(b):
        return [b[0] - x0, b[1] - y0, b[2], b[3]]
    sheet = dict(target=f'{t["name"]} (group {group})', kind=kind, panel_size=panel[2:], opens_in=where_short,
                 where=where, panel_in_pane=panel[:2], pane_size=list(S.pane),
                 regions=[dict(n=f'R{n}', role=r['words'], rect=rel(r['rect']), drawn=r['drawn'])
                          for n, r in enumerate(regions, 1)],
                 strings=[dict(n=f'T{n}', role=s['role'], lines=s['lines'], letters=rel(s['ink']),
                               colour=s['colour'], line_pitch=s['pitch'], lines_aligned=s['lines_aligned'],
                               emphasised=s['emphasised'])
                          for n, s in enumerate(texts, 1)], sheet_version=2)
    # pictures: the real panel drawn by iface.py from the state TSPS showed; checked against the TSPS capture
    a, _, _ = S.images()
    crop = a[panel[1]:panel[1] + panel[3], panel[0]:panel[0] + panel[2]].astype(np.uint8)
    check = None
    tp = HERE / t['tsps']
    if tp.exists():
        cap = np.asarray(Image.open(tp).convert('RGB')).astype(int)
        root = S.lay[group << 16]
        org = t.get('tsps_origin') or root['abs']
        if t['pane'] == 'hud':
            org = [panel[0] - 1, panel[1] - 1]
        ox, oy = panel[0] - org[0], panel[1] - org[1]
        sub = cap[oy:oy + panel[3], ox:ox + panel[2]]
        if sub.shape[:2] == crop.shape[:2]:
            check = int((sub != crop).any(-1).sum())
    Image.fromarray(crop).save(tdir / 'real-1x.png')
    Image.fromarray(crop).resize((panel[2] * 3, panel[3] * 3), Image.NEAREST).save(tdir / 'real-3x.png')
    pic = crop
    if check is not None:
        pic = sub.astype(np.uint8)  # the TSPS capture itself (it shows models iface.py does not draw)
        Image.fromarray(pic).save(tdir / 'real-tsps-1x.png')
    if kind == 'picture':
        Image.fromarray(pic).save(outdir / 'panel-1x.png')
        Image.fromarray(pic).resize((panel[2] * 3, panel[3] * 3), Image.NEAREST).save(outdir / 'panel-3x.png')
    wire(panel, regions, texts, outdir / 'wire.png')
    (outdir / 'sheet.json').write_text(json.dumps(sheet, indent=1))
    md = [f'# Sheet: rebuild {t["name"]} ({kind} sheet)', '',
          f'- Panel: **{panel[2]} x {panel[3]} px** (the rectangle its pixels cover).',
          f'- Opens in: {where}.',
          '- All rectangles below are relative to the panel\'s top-left, in pixels: x, y, width, height.',
          '- `wire.png`: the same regions and strings drawn at 2x (wire colours mean kinds of region, not colours of'
          ' the panel).']
    if kind == 'picture':
        md.append('- `panel-1x.png`, `panel-3x.png`: the panel as the TSPS client drew it (a crop of its screenshot'
                  + (f'; it differs from iface.py\'s drawing in {check} pixels, all inside the model rectangle, which'
                     ' iface.py does not draw' if check else '; iface.py draws it identically') + ').')
    else:
        md.append('- No picture of the panel: this is a RULES sheet. Sprites, fonts, colours, transparencies and '
                  'construction come from INTERFACES.md, not from here.')
    md += ['', '## Regions', '', '`drawn` is the order the regions are drawn in (1 = first; a region counts from its '
           'first-drawn part, so a box whose contents are drawn before its frame comes after them); where regions '
           'overlap, the later one is on top.', '',
           '| # | region (role in words) | x | y | w | h | drawn |', '|---|---|---|---|---|---|---|']
    for r in sheet['regions']:
        md.append(f'| {r["n"]} | {r["role"]} | {r["rect"][0]} | {r["rect"][1]} | {r["rect"][2]} | {r["rect"][3]} | '
                  f'{r["drawn"]} |')
    md += ['', '## Strings in reading order', '',
           '`letters` is the box the letters (and their shadow, if any) cover, not the text component\'s box. Lines are '
           'as drawn (wrapped or broken). `colour` is the text\'s own colour by its name in the palette of INTERFACES.md '
           '(section 12). `line pitch` is the distance in pixels from one line\'s baseline to the next and `lines` '
           'how the lines are aligned to each other (multi-line texts only). "Emphasised" parts are drawn in another '
           'colour than the rest of that text; the colour\'s palette name follows each part in brackets.',
           '', '| # | role | text (lines separated by " / ") | letters x, y, w, h | colour | line pitch | lines | '
           'emphasised parts |', '|---|---|---|---|---|---|---|---|']
    for s in sheet['strings']:
        L = s['letters']
        md.append(f'| {s["n"]} | {s["role"]} | {" / ".join(s["lines"])} | {L[0]}, {L[1]}, {L[2]}, {L[3]} | '
                  f'{s["colour"]} | {s["line_pitch"] or "-"} | {s["lines_aligned"] or "-"} | '
                  f'{"; ".join(s["emphasised"]) or "-"} |')
    if not sheet['strings']:
        md.append('| - | - | (no visible text) | - | - | - | - | - |')
    md += ['', '## Legend of region words (INTERFACES.md v2)', '',
           '- titled steel-border panel / stone-border panel: the panel frame with its title bar (recipes 2.1, 2.3).',
           '- steel-border inner box: corners and edges without a title (2.2). "no backing of its own": it draws only'
           ' its border.',
           '- thin-border box (with a header band; its line at ... of the box): recipe 2.5; the line given is the 3px '
           'band strip you can see, relative to the box\'s top-left.',
           '- dark / light / medium / beige bevel box, grey / red tile box: recipes 2.4, 2.6, 2.7. "button (...)": '
           'such a box with a text inside it.',
           '- two-tone outline (a outside, b inside), outline box (1px, colour), filled box, translucent highlight: '
           'rectangles (section 4), colours by palette name; "transparency t" is the component\'s transparency 0-255.',
           '- box built of sprite pieces: a box drawn from corner, edge and middle sprites that are NOT one of the '
           'families of INTERFACES.md section 2 (undecidable from this sheet).',
           '- backing, light well, dark well, beige well: tiled backgrounds (section 3); "translucent: transparency t"'
           ' gives its transparency; "mirrored left-right" means drawn flipped horizontally.',
           '- icon X: a single sprite picture; the same letter means the same picture.',
           '- bar (base colour), with a filled part (colour) w x h from its left end: section 7.',
           '- rule line (colour, width): section 4.',
           '- model: a 3D model view, not scored (the gate leaves its rectangle out).',
           '', '## Roles of strings', '',
           '- title: in a titled frame\'s title bar. heading: in a header band. button: inside a button. body: a text'
           ' over 40 characters. value: numbers. label: everything else.']
    (outdir / 'sheet.md').write_text('\n'.join(md) + '\n', encoding='utf-8')
    # teacher-only
    subtree = []
    for i in S.order:
        row = dict(S.byid[i]); p = S.lay[i]
        row.update(abs=p['abs'], size=p['size'], clip=p['clip'], hidden=p['hidden'],
                   parent=-1 if p['parent'] is None else p['parent'])
        subtree.append(row)
    (tdir / 'state.jsonl').write_text('\n'.join(json.dumps(r) for r in subtree) + '\n')
    ctrl = R.scene_to_components(S, group)
    (tdir / 'control.json').write_text(json.dumps(dict(components=ctrl, pane=S.kind)))
    index = {i: n for n, i in enumerate(S.order)}
    (tdir / 'classify.json').write_text(json.dumps(dict(
        regions=[dict(n=f'R{n}', family=r['family'], words=r['words'], rect=r['rect'],
                      members=[index[m] for m in r['members']]) for n, r in enumerate(regions, 1)],
        texts=[dict(n=f'T{n}', role=s['role'], text=s['text'], ink=s['ink'], member=index[s['id']])
               for n, s in enumerate(texts, 1)]), indent=1))
    (tdir / 'meta.json').write_text(json.dumps(dict(group=group, name=t['name'], state=t['state'], pane=S.kind,
                                                    pane_size=list(S.pane), panel=panel, tsps_check_differ=check),
                                               indent=1))
    print(f'{group} {kind}: panel {panel[2]}x{panel[3]} at {panel[:2]}, {len(regions)} regions, {len(texts)} strings, '
          f'picture vs TSPS capture differ {check}; -> {outdir}')


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument('group', type=int); ap.add_argument('outdir')
    ap.add_argument('--kind', choices=['rules', 'picture'], required=True)
    a = ap.parse_args()
    make(a.group, a.outdir, a.kind)


if __name__ == '__main__':
    main()
