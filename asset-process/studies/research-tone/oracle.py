"""Step 1 experiments that isolate causes by building hybrid meshes (Teacher-side: they read the real mesh).

usage: oracle.py NAME [NAME..]
  X1  OUR geometry, REAL paint by nearest face          -> what is left is light-from-shape + face resolution
  X2  REAL geometry, OUR paint by nearest face          -> what is left is paint (colour, place, detail)
  X3  OUR geometry, free-palette COMPENSATING paint: each face gets the lightness that makes its lit tone equal the
      nearest real face's lit tone under OUR light (the best any per-face repaint could do on this geometry)
  X4  as X3 but restricted to the sheet's palette (nearest palette colour of the same hue family)
  X5  REAL geometry and paint with every translucent face made opaque in BOTH (ours as delivered, opaque)
Each is gated with the unmodified gate; tone lines are printed.
"""
import sys, pathlib; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import json
import numpy as np
import tl, transfer
import light_npc

def vertex_ln(a, ambient, contrast):
    F = [(int(f[0]), int(f[1]), int(f[2]), int(c), int(t), int(al), -1) for f, c, t, al in zip(a['F'], a['C'], a['T'], a['A'])]
    V = [tuple(int(x) for x in v) for v in a['V']]
    ln = light_npc.vertex_light(V, F, 64 + ambient, 850 + contrast * 5)
    return np.array([np.nan if x is None else x for x in ln], float)
def face_ln(a, ambient, contrast):
    return np.nanmean(vertex_ln(a, ambient, contrast)[a['F']], axis=1)

def run(name):
    b = tl.BOSSES.get(name) or tl.MIDS[name]; npc = b['npc']
    mr, kv = tl.real_mesh(npc); dr = tl.real_def(npc); mo, jo = tl.ours_mesh(b['ours'])
    ar, ao = tl.arrays(mr), tl.arrays(mo); amb, con = dr['ambient'], dr['contrast']
    o2r, dist = transfer.nearest(ar, ao); r2o, dist2 = transfer.nearest(ao, ar)
    print(f'\n=== {name}: nearest-face distance, ours->real median {np.median(dist[o2r >= 0]):.1f} units, 90th {np.quantile(dist[o2r >= 0], .9):.1f}; real->ours median {np.median(dist2[r2o >= 0]):.1f}, 90th {np.quantile(dist2[r2o >= 0], .9):.1f}')
    res = {}
    def g(tag, json_path):
        v, vals, rows = tl.gate(npc, json_path, f'oracle-{name}-{tag}', tl.RENDER / f'research-tone-oracle-{name}-{tag}'); tl.show_tones(f'{name} {tag}', v, vals); res[tag] = dict(verdict=v, tones=tl.tone_lines(vals))
    g('X0-as-delivered', b['ours'])
    m = tl.clone(mo); m['colors'] = [int(ar['C'][o2r[i]]) if o2r[i] >= 0 else c for i, c in enumerate(mo['colors'])]
    g('X1-ourgeom-realpaint', tl.write_variant(f'{name}-X1', m, jo, 'oracle'))
    m = tl.clone(mr); m['colors'] = [int(ao['C'][r2o[i]]) if r2o[i] >= 0 else c for i, c in enumerate(mr['colors'])]
    g('X2-realgeom-ourpaint', tl.write_variant(f'{name}-X2', m, dr, 'oracle'))
    lr, lo = face_ln(ar, amb, con), face_ln(ao, amb, con)
    pal = sorted(set(int(c) for c, d in zip(ar['C'], tl.drawn(ar)) if d))
    cols3, cols4 = [], []
    for i, c in enumerate(mo['colors']):
        j = o2r[i]
        if j < 0 or not np.isfinite(lo[i]) or lo[i] < 4: cols3.append(c); cols4.append(c); continue
        cr = int(ar['C'][j]); target = (cr & 127) * lr[j] / 128.0
        l = int(np.clip(round(target * 128.0 / lo[i]), 2, 126)); cols3.append((cr & ~127) | l)
        best = min(pal, key=lambda p: (abs((p & 127) - l) + (0 if (p >> 7) == (cr >> 7) else 4 * min(abs((p >> 10) - (cr >> 10)), 64 - abs((p >> 10) - (cr >> 10))) + 3 * abs(((p >> 7) & 7) - ((cr >> 7) & 7)))))
        cols4.append(best)
    m = tl.clone(mo); m['colors'] = cols3; g('X3-ourgeom-compensated-free', tl.write_variant(f'{name}-X3', m, jo, 'oracle'))
    m = tl.clone(mo); m['colors'] = cols4; g('X4-ourgeom-compensated-palette', tl.write_variant(f'{name}-X4', m, jo, 'oracle'))
    (tl.WORK / 'oracle').mkdir(exist_ok=True); (tl.WORK / 'oracle' / f'{name}.json').write_text(json.dumps(res, indent=1))
if __name__ == '__main__':
    for n in sys.argv[1:]: run(n)
