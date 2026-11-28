"""One picture per boss: the real boss, known damage, and the rebuild, side by side under the gate's front camera, with the
gate's standing tone under each. usage: board_damage.py"""
import sys, pathlib; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import json
import numpy as np
from PIL import Image, ImageDraw
import tl
from tl import L
SETS = {7541: [('real', None), ('all 188 detail islands removed', 'damage/7541-islands100.json'), ('same paint, diagonals re-split', 'damage/7541-resplit.json'), ('vertices +/-3 units', 'damage/7541-jitter3.json'),
               ('vertices +/-6 units', 'damage/7541-jitter6.json'), ('rebuild (second author)', tl.BOSSES['tekton2']['ours'])],
        11278: [('real', None), ('all 123 detail islands removed', 'damage/11278-islands100.json'), ('same paint, diagonals re-split', 'damage/11278-resplit.json'), ('vertices +/-1 unit', 'damage/11278-jitter1.json'),
                ('vertices +/-2 units', 'damage/11278-jitter2.json'), ('rebuild', tl.BOSSES['nex']['ours'])]}
for npc, items in SETS.items():
    seqs = L.sequences(npc); cam = tl.camera(npc); ents = [L.entry_real(npc, seqs)] + [tl.entry(tl.WORK / p if isinstance(p, str) else p, seqs) for _, p in items[1:]]
    for row in ('bind', 'walk:8' if npc == 7541 else 'walk:9'):
        cells, _ = tl.draw(ents, [row], cam, 192); cs = cells[0]; ys, xs = np.nonzero(np.any([tl.mask(c) for c in cs], 0)); y0, y1, x0, x1 = max(ys.min() - 4, 0), ys.max() + 5, max(xs.min() - 4, 0), xs.max() + 5; z = 3; tiles = []
        for (cap, _), c in zip(items, cs):
            t = Image.fromarray(c[y0:y1, x0:x1]).resize(((x1 - x0) * z, (y1 - y0) * z), Image.NEAREST); cv = Image.new('RGB', (t.width, t.height + 34), (18, 22, 28)); cv.paste(t, (0, 34)); d = ImageDraw.Draw(cv)
            d.text((4, 3), cap, fill=(240, 240, 240)); d.text((4, 18), 'tone in this cell: ' + ('0' if cap == 'real' else f'{tl.tone(cs[0], c):.1f}') + '  (pass limit 8)', fill=(255, 210, 120)); tiles.append(cv)
        b = Image.new('RGB', (sum(t.width for t in tiles) + 4 * len(tiles), tiles[0].height), (18, 22, 28)); x = 0
        for t in tiles: b.paste(t, (x, 0)); x += t.width + 4
        out = tl.RENDER / 'research-tone-gate-check'; out.mkdir(exist_ok=True); b.save(out / f'{npc}-{row.replace(":", "")}.png'); print(out / f'{npc}-{row.replace(":", "")}.png')
