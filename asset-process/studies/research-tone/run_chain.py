"""Chains of author-side steps on a boss rebuild, each stage gated.
usage: run_chain.py NAME TAG step [step ...]
steps:  floor            clamp vertices below the sheet's floor (height 0) to it, and above its top to the top
        nf[:R[:S]]       normalfit.fit, radius R (default 6), sweeps S (default 12);  nfflip = flips only
        pf:VARIANT       paintfit, VARIANT as in run_paintfit.py (e.g. pf:both-s40-alpha)
The gate is run on the result of every step (unmodified gate, storage redirected)."""
import sys, pathlib; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import json, time
import numpy as np
import tl, paintfit, normalfit
from run_paintfit import SHEETS
def floor(src, sheet, name):
    m, j = tl.ours_mesh(src); info = json.loads((sheet / 'sheet.json').read_text()); lo, hi = info['bounds_min'], info['bounds_max']
    V = [(int(np.clip(x, lo[0], hi[0])), int(np.clip(y, lo[1], hi[1])), int(np.clip(z, lo[2], hi[2]))) for x, y, z in m['vertices']]
    n = sum(1 for a, b in zip(V, m['vertices']) if tuple(a) != tuple(b)); m['vertices'] = V
    print(f'{name}: {n} vertices clamped into the sheet bounds {lo} .. {hi}')
    return tl.write_variant(name, m, j, 'chain')
def run(name, tag, steps, gate_each=True):
    b = tl.BOSSES[name]; npc = b['npc']; sheet = SHEETS[npc]; cur = b['ours']; out = tl.WORK / 'chain'; res = []
    for k, st in enumerate(steps):
        nm = f'{name}-{tag}-{k + 1}-{st.replace(":", "_")}'; t0 = time.time()
        if st == 'floor': cur = floor(cur, sheet, nm)
        elif st.startswith('nf'):
            p = st.split(':'); cur, rep = normalfit.fit(cur, sheet, out, nm, float(p[1]) if len(p) > 1 else 6, int(p[2]) if len(p) > 2 else 12, True, p[0] != 'nfflip', quiet=True)
        elif st.startswith('pf:'):
            parts = st[3:].split('-'); mode = parts[0]; source = next((p for p in parts if p in ('pose', 'ortho', 'bind')), 'both'); splits = next((int(p[1:]) for p in parts if p[0] == 's' and p[1:].isdigit()), 0)
            cur, rep = paintfit.run(cur, sheet, out, mode, splits, 'alpha' in parts, 'fill' in parts, 'all', 1, nm, source)
        else: raise SystemExit('unknown step ' + st)
        if gate_each or k == len(steps) - 1:
            v, vals, rows = tl.gate(npc, cur, f'chain-{nm}', tl.RENDER / f'research-tone-chain-{nm}'); tl.show_tones(nm, v, vals); res.append((nm, v, vals))
        print('   seconds', round(time.time() - t0), flush=True)
    return cur, res
if __name__ == '__main__':
    run(sys.argv[1], sys.argv[2], sys.argv[3:])
