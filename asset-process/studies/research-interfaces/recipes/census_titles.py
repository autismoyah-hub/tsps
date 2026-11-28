"""census_titles.py: the title text of titled steel / stone frames and the texts of thin boxes with a header band,
relative to their box (held-out excluded). Prints counts by (font, colour, shadow, alignment, box)."""
import sys, collections
from pathlib import Path
sys.path.insert(0, str(Path(__file__).resolve().parent))
import common as K
import census_frames as F

FN = {494: 'p11', 495: 'p12', 496: 'b12', 497: 'q8'}
T = collections.Counter(); TG = collections.defaultdict(set)
TH = collections.Counter(); THG = collections.defaultdict(set)
for g, rows, by in K.all_states():
    vis = [r for r in rows if r['visible'] and r['kind'] == 4 and r['text'] and r['font'] >= 0]
    for name in ('steel', 'stone'):
        for inst in F.find_instances(rows, by, F.FAMILIES[name]):
            if not any(m['sprites'][0] in (2546, 828) for m in inst['members']):
                continue
            x, y, W, H = inst['box']
            for r in vis:
                rx, ry = r['abs'][0] - x, r['abs'][1] - y
                if 0 <= rx <= 12 and 0 <= ry <= 12 and r['size'][1] <= 30:
                    box = (rx, ry, 'W' + str(r['size'][0] - W), r['size'][1])
                    k = (name, FN.get(r['font'], r['font']), f"{r['colors'][0]:06x}", int(r['text_shadow']),
                         tuple(r['text_alignment']), box)
                    T[k] += 1; TG[k].add(g)
    for inst in F.find_instances(rows, by, F.FAMILIES['thin']):
        band = next((m for m in inst['members'] if m['sprites'][0] == 995), None)
        if band is None:
            continue
        x, y, W, H = inst['box']
        line = band['abs'][1] - y + 17
        for r in vis:
            rx, ry = r['abs'][0] - x, r['abs'][1] - y
            if 0 <= rx < W and 0 <= ry < H:
                part = 'header' if ry < line else 'body'
                k = (part, FN.get(r['font'], r['font']), f"{r['colors'][0]:06x}", int(r['text_shadow']),
                     tuple(r['text_alignment']), 'top' if ry in (0, line + 3) else ry - (0 if part == 'header' else line + 3),
                     'to line' if part == 'header' and r['size'][1] == line + 3 else r['size'][1], r['line_height'])
                TH[k] += 1; THG[k].add(g)
print('## titles of titled frames (frame, font, colour, shadow, align, box rel. frame): uses, groups')
for k, v in T.most_common(14):
    print('  ', k, v, len(TG[k]))
print('## texts in thin boxes with a header band (part, font, colour, shadow, align, top, height, lh): uses, groups')
for k, v in TH.most_common(14):
    print('  ', k, v, len(THG[k]))
