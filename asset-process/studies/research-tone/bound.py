"""Teacher-side BOUND: the best a per-face repaint of a rebuild's mesh (geometry untouched) can score, found by fitting
each face's palette colour to the gate's OWN cells (its two cameras, its sample frames) under the mesh's own light.
Not a tool for authors (it fits the test); it answers "can paint alone pass on this geometry?".
usage: bound.py NAME [NAME..]     variants: all cells; rest cells only (the best possible 'standing picture')"""
import sys, pathlib; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import json
import numpy as np
import tl, paintfit
from sheet_boss import Sheet
G = {7541: tl.B / 'sheets-teacher-only' / 'tekton-gatecells', 11278: tl.B / 'sheets-teacher-only' / 'nex-gatecells'}
def run(name):
    b = tl.BOSSES[name]; npc = b['npc']; sd = G[npc]; sheet = Sheet(sd); m, j = tl.ours_mesh(b['ours']); a = tl.arrays(m)
    for tag, bind_only in (('allcells', False), ('restcells', True)):
        acc = paintfit.Acc(a, sheet, j.get('ambient', 0), j.get('contrast', 0)); paintfit.feed_pose(acc, sd, b['ours'], 'lit', bind_only, 'bound-ids', 1.0)
        r = paintfit.decide(acc, 'lit'); dr = tl.drawn(a); mm = tl.clone(m); new = a['C'].copy(); s = r['seen'] & dr; new[s] = r['colour'][s]; mm['colors'] = [int(c) for c in new]
        p = tl.write_variant(f'{name}-bound-{tag}', mm, j, 'bound'); v, vals, rows = tl.gate(npc, p, f'bound-{name}-{tag}'); tl.show_tones(f'{name} bound {tag}', v, vals)
if __name__ == '__main__':
    for n in sys.argv[1:]: run(n)
