"""assemble.py: ARENAS.src.md + the generated tables -> ARENAS.md; and the small floor-comparison board.
Run after measure.py, table.py and rules.py. Markers in the source: <<OVERVIEW>>, <<RULECOUNTS>>, <<TABLES>>."""
import json, pathlib, subprocess, sys
from PIL import Image, ImageDraw
HERE = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))
import rules as RU
D = json.loads((HERE / 'data' / 'arenas.json').read_text()); bb = RU.load_bands(); B = bb['bands']

ov = ['| arena | where (world tiles, plane) | floor tiles | box | outline | boss (tiles a side) | boss stands | bounded mostly by | found by |', '|---|---|---|---|---|---|---|---|---|']
for k, m in D.items():
    b = m['bbox']
    ov.append(f"| {m['name']} (`{k}`) | ({b[0]},{b[1]})..({b[2]},{b[3]}), plane {m['plane']} | {m['area']} | {m['w']} x {m['h']} | {m['outline']} | {m['boss_side']} | {m['boss_place']} | {m['edge_main']} | {m['src']}: {m['where']} |")

f = lambda v: '-' if v is None else (str(int(v)) if float(v).is_integer() else f'{v:.2f}')
rc = ['| rule | holds for | limits (real p25 / median / p75) | does not hold for |', '|---|---|---|---|']
for rid, r in RU.counts(D, B).items():
    n = len(r['holds']) + len(r['outside'])
    tests = next(t for i, _, t in RU.RULES if i == rid)
    lim = '; '.join(f"`{k}` {f(RU.limit(lo, k, B))} .. {f(RU.limit(hi, k, B))} ({f(B[k]['p25'])} / {f(B[k]['med'])} / {f(B[k]['p75'])})" for k, lo, hi, aim in tests if k in B)
    rc.append(f"| {rid} | {len(r['holds'])} of {n}" + (f" ({len(r['na'])} n/a)" if r['na'] else '') + f" | {lim} | {', '.join(r['outside']) or '-'} |")

src = (HERE / 'ARENAS.src.md').read_text(encoding='utf-8')
tables = (HERE / 'data' / 'tables.md').read_text(encoding='utf-8')
out = src.replace('<<OVERVIEW>>', '\n'.join(ov)).replace('<<RULECOUNTS>>', '\n'.join(rc)).replace('<<TABLES>>', tables)
(HERE / 'ARENAS.md').write_text(out, encoding='utf-8', newline='\n')
print('ARENAS.md', len(out.splitlines()), 'lines')

R = HERE.parents[1] / 'render'
rows = [('boards + star dial, dark masonry (r1): boss delta E 27.8, brightness 13.9 off the floor', R / 'research-arenas-warden-r1'),
        ('armoury flags + brass ring (r1-flags): boss delta E 2.9, brightness 7.1 off the floor: the grey giant sinks into the grey floor', R / 'research-arenas-warden-r1-flags')]
if all((p / 'played.png').exists() for _, p in rows):
    im = Image.new('RGB', (1280, 2 * (450 + 20)), (24, 24, 28)); dr = ImageDraw.Draw(im)
    for r, (label, p) in enumerate(rows):
        for c, v in enumerate(('played', 'top')):
            im.paste(Image.open(p / f'{v}.png').convert('RGB').resize((640, 450), Image.LANCZOS), (c * 640, r * 470 + 20))
        dr.text((6, r * 470 + 4), label, fill=(255, 230, 120))
    im.save(HERE / 'boards' / 'board-warden-floors.jpg', quality=92); print('board-warden-floors.jpg')
