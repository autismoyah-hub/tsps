"""Step 3 proof: paint-fit on the three boss rebuilds' meshes as delivered (geometry untouched), gated.
usage: run_paintfit.py NAME [variant ...]
variant = mode[-pose|-ortho|-bind][-six][-old][-alpha][-fill][-sN]
  mode: paint | lit | both;  source: default both (ortho + pose), or pose / ortho / bind;
  six = only the six axis views of the ortho set; old = six views at the first sheet's resolution (about 1.25 px per
  unit) and no pose set: what the FIRST sheet gave;  sN = apply N cuts."""
import sys, pathlib; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import json, time
import tl, paintfit
SHEETS = {7541: tl.B / 'sheets' / 'tekton', 11278: tl.B / 'sheets' / 'nex'}
def one(name, variant, src=None, tag=None, npc=None):
    b = tl.BOSSES.get(name); npc = npc or b['npc']; parts = variant.split('-'); mode = parts[0]; sheet = SHEETS[npc]
    px = json.loads((sheet / 'sheet.json').read_text())['pixels_per_unit']
    stride = max(1, round(px / 1.25)) if 'old' in parts else 1; views = 'six' if ('six' in parts or 'old' in parts) else 'all'
    source = 'ortho' if 'old' in parts else next((p for p in parts if p in ('pose', 'ortho', 'bind')), 'both')
    splits = next((int(p[1:]) for p in parts if p[0] == 's' and p[1:].isdigit()), 0)
    tag = tag or f'{name}-{variant}'
    path, rep = paintfit.run(src or b['ours'], sheet, tl.WORK / 'paintfit', mode, splits, 'alpha' in parts, 'fill' in parts, views, stride, tag, source)
    v, vals, rows = tl.gate(npc, path, f'paintfit-{tag}', tl.RENDER / f'research-tone-paintfit-{tag}'); tl.show_tones(tag, v, vals)
    return path, rep, v, vals
if __name__ == '__main__':
    name = sys.argv[1]
    for variant in sys.argv[2:]:
        t = time.time(); one(name, variant); print('   seconds', round(time.time() - t), flush=True)
