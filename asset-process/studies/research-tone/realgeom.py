"""Is the SHEET the limit, or the GEOMETRY? Teacher-side experiment with the author-side tool:
the real boss's own geometry, every face repainted ONE flat colour (its paint thrown away), then painted back by
paintfit.py from a sheet alone. If the tool restores the tone on the real faces, the sheet carries the information and
the tool can place it; what stops the rebuilds is then where their faces are.
usage: realgeom.py NPC_ID [variant ...]    variants as run_paintfit.py (paint, paint-old, paint-ortho, paint-six, lit ..)
       realgeom.py NPC_ID jitter:N variant   the same after moving every vertex up to N units on each axis (seeded)
"""
import sys, pathlib; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import json, random
import numpy as np
import tl, paintfit
from run_paintfit import SHEETS
def run(npc, variants):
    m, kv = tl.real_mesh(npc); d = tl.real_def(npc); a = tl.arrays(m); dr = tl.drawn(a); jit = 0
    pal = sorted(set(a['C'][dr].tolist())); flat = int(np.bincount(a['C'][dr]).argmax())
    for variant in variants:
        if variant.startswith('jitter:'): jit = int(variant.split(':')[1]); continue
        mm = tl.clone(m); mm['colors'] = [flat if x else c for c, x in zip(m['colors'], dr)]
        if jit:
            rng = random.Random(239); mm['vertices'] = [tuple(c + rng.randint(-jit, jit) for c in v) for v in m['vertices']]
        tag = f'realgeom-{npc}-j{jit}-{variant}'; src = tl.write_variant(tag + '-blank', mm, d, 'realgeom')
        parts = variant.split('-'); mode = parts[0]; px = json.loads((SHEETS[npc] / 'sheet.json').read_text())['pixels_per_unit']
        stride = max(1, round(px / 1.25)) if 'old' in parts else 1; views = 'six' if ('six' in parts or 'old' in parts) else 'all'
        source = 'ortho' if 'old' in parts else next((p for p in parts if p in ('pose', 'ortho', 'bind')), 'both')
        path, rep = paintfit.run(src, SHEETS[npc], tl.WORK / 'realgeom', mode, 0, False, 'fill' in parts, views, stride, tag, source, quiet=True)
        mo, _ = tl.ours_mesh(path); same = np.array(mo['colors'])[dr] == a['C'][dr]
        area = np.linalg.norm(np.cross(a['V'][a['F'][:, 1]] - a['V'][a['F'][:, 0]], a['V'][a['F'][:, 2]] - a['V'][a['F'][:, 0]]), axis=1)[dr]
        v, vals, rows = tl.gate(npc, path, tag)
        print(f"{tag}: faces given their real colour {same.mean():.3f} (by surface {area[same].sum() / area.sum():.3f}); unseen {rep['faces_unseen']} faces ({100 * rep['unseen_area_share']:.1f}% of surface); contested {100 * rep['contested_area_share']:.1f}% of surface")
        tl.show_tones(tag, v, vals)
if __name__ == '__main__':
    run(int(sys.argv[1]), sys.argv[2:])
