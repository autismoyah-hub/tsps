"""Reproduce the gate on the three boss rebuilds as delivered, and check the ID / LIGHT pictures on the real mesh.
usage: baseline.py [gate] [idcheck]"""
import sys, time, json, pathlib; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np
import tl
from tl import L
a = sys.argv[1:]
if 'gate' in a:
    for k, b in list(tl.BOSSES.items()) + (list(tl.MIDS.items()) if 'mids' in a else []):
        t = time.time(); v, vals, rows = tl.gate(b['npc'], b['ours'], f'base-{k}', tl.RENDER / f'research-tone-gate-base-{k}')
        tl.show_tones(k, v, vals); print('   seconds', round(time.time() - t))
if 'idcheck' in a:
    for npc in (7541, 11278):
        m, kv = tl.real_mesh(npc); d = tl.real_def(npc); seqs = L.sequences(npc); cam = tl.camera(npc)
        ar = tl.arrays(m); nf = len(ar['F'])
        ceil = tl.write_variant(f'real-{npc}', m, d, 'ceiling')
        ids = tl.id_variants(f'real-{npc}', m, d)
        ents = [L.entry_real(npc, seqs)] + [tl.entry(p, seqs) for p in [ceil] + ids]
        for yaw in (192, 1216):
            cells, text = tl.draw(ents, ['bind'], cam, yaw, tl.RENDER / 'research-tone-work' / f'idcheck-{npc}-{yaw}.png')
            real, ce, i0, i1, li = cells[0]
            print(npc, yaw, 'ceiling differs from real on', int((real != ce).any(2).sum()), 'pixels; drawn', int(tl.mask(real).sum()))
            c0, c1 = tl.decode_codes(i0), tl.decode_codes(i1); fid = np.where((c0 >= 0) & (c1 >= 0), c0 + tl.NCODE * c1, -1)
            mk = tl.mask(real); print('   id mask == real mask:', bool((tl.mask(i0) == mk).all()), '| undecoded pixels', int(((fid < 0) & tl.mask(i0)).sum()), '| ids beyond face count', int((fid >= nf).sum()), '| distinct faces seen', len(np.unique(fid[fid >= 0])))
            ln = tl.decode_light(li); print('   light: undecoded', int((np.isnan(ln) & tl.mask(li)).sum()), 'range', np.nanmin(ln), np.nanmax(ln))
            paint = np.where(fid >= 0, ar['C'][np.clip(fid, 0, nf - 1)], -1)
            g = tl.synth(paint, ln); ok = np.isfinite(g) & mk
            err = np.abs(g - tl.grey(real))[ok]
            alpha = np.where(fid >= 0, ar['A'][np.clip(fid, 0, nf - 1)], 0) > 0
            print(f'   synth(real paint, real light) vs real picture: mean abs grey {err.mean():.3f}; on opaque-front pixels {np.abs(g - tl.grey(real))[ok & ~alpha].mean():.3f} ({int((ok & ~alpha).sum())} px); on translucent-front pixels {np.abs(g - tl.grey(real))[ok & alpha].mean():.3f} ({int((ok & alpha).sum())} px)')
