"""Which faces does each pose show? For the real boss (Teacher-side) and for a rebuild: pixels per face in the pose
set's rest cells against its stand / walk / attack cells and its death cells. usage: layers.py NAME"""
import sys, pathlib; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import json, collections
import numpy as np
import tl, poseset
from tl import L
from run_paintfit import SHEETS
def vis(mesh, base_def, name, sheet):
    info = json.loads((sheet / 'pose.json').read_text()); seqs = info['sequences']; cam = dict(zoom=info['zoom'], aim=info['aim'], cell=tuple(info['cell']))
    var = poseset.layer_variants(name, mesh, base_def, 'layers'); ents = [tl.entry(p, seqs) for p in var]; nf = len(mesh['faces']); out = collections.defaultdict(lambda: np.zeros(nf))
    for yaw, pitch, rws in info['cameras']:
        if pitch != 128: continue
        for row, cs in zip(rws, poseset.draw_layers(ents, rws, cam, yaw, pitch)):
            fA, fO, ln = poseset.decode(cs, nf); k = 'bind' if row == 'bind' else 'death' if row.startswith('death') else 'live'
            out[k] += np.bincount(fA[fA >= 0], minlength=nf) / (1 if k == 'bind' else 6 if k == 'live' else 4)
    return out
def table(tag, mesh, v):
    a = tl.arrays(mesh); dr = tl.drawn(a); b, l, d = v['bind'], v['live'], v['death']
    cls = np.where(~dr, 'undrawn', np.where((b > 4) & (l < 0.15 * b), 'rest only', np.where((b < 0.15 * l) & (l > 4), 'live only', np.where((b <= 4) & (l <= 4) & (d > 4), 'death only', np.where((b <= 4) & (l <= 4), 'never', 'both')))))
    print(f'{tag}: faces by where they show (mean pixels per cell over 8 directions)')
    for c in ('both', 'rest only', 'live only', 'death only', 'never', 'undrawn'):
        s = cls == c
        if not s.any(): continue
        cols = collections.Counter(a['C'][s].tolist()).most_common(4); lit = np.mean([(c_ & 127) for c_ in a['C'][s]])
        print(f"   {c:10s} {int(s.sum()):5d} faces; rest px {b[s].sum():8.0f}, live px {l[s].sum():8.0f}, death px {d[s].sum():8.0f}; translucent {int((a['A'][s] > 0).sum())}; depth-offset {int((a['D'][s] != 0).sum())}; mean paint lightness {lit:.0f}; top colours {[(tl.hsl(c_), n) for c_, n in cols]}")
    return cls
if __name__ == '__main__':
    if sys.argv[1] == '--mine':                                   # author-side: layers.py --mine DEF.json SHEET_DIR (reads no real mesh)
        mo, jo = tl.ours_mesh(sys.argv[2]); sheet = pathlib.Path(sys.argv[3]); table('MINE', mo, vis(mo, jo, pathlib.Path(sys.argv[2]).stem, sheet))
        print('the real boss (sheet.json, faces_by_pose):', json.loads((sheet / 'sheet.json').read_text()).get('faces_by_pose')); raise SystemExit(0)
    name = sys.argv[1]; b = tl.BOSSES[name]; npc = b['npc']; sheet = SHEETS[npc]
    mr, kv = tl.real_mesh(npc); cr = table('REAL', mr, vis(mr, tl.real_def(npc), f'real-{npc}', sheet))
    mo, jo = tl.ours_mesh(b['ours']); co = table('OURS ' + name, mo, vis(mo, jo, f'ours-{name}', sheet))
    np.save(tl.WORK / f'layers-{name}-real.npy', cr); np.save(tl.WORK / f'layers-{name}-ours.npy', co)
