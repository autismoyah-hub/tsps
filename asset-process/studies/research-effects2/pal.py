"""pal.py EFFECT..: palette (h, s, l, share), alpha values, box, definition light of real effects (id:N) or objects (loc:N)."""
import sys, pathlib
sys.dont_write_bytecode = True; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import census, objlib, objcensus
from fxlib import *
for e in sys.argv[1:]:
    k, v = e.split(':')
    if k == 'id':
        d = decode_spot(spots()[int(v)]); m = spot_model(d); st = census.model_stats(m)
        s = al.sequence(d['sequence']) if d['sequence'] >= 0 else None
        print(e, names(8).get(int(v)), 'scale', d['scale'], 'amb', d['ambient'], 'con', d['contrast'], 'seq', d['sequence'], 'holds', s and s['lengths'][:30], 'loop', s and s['loop_frames'])
    else:
        d = objlib.obj(int(v)); m = objcensus.combined(d, objlib.shape_for(d)); st = census.model_stats(m)
        s = al.sequence(d['anim'])
        print(e, names(6).get(int(v)), 'scale', d['scale'], 'amb', d['ambient'], 'con', d['contrast'], 'seq', d['anim'], 'holds', s['lengths'][:40], 'loop', s['loop_frames'], 'size', d['size'])
    print('   faces', st['faces'], 'box lo', st['lo'], 'hi', st['hi'], 'alpha', st['alpha_values'], 'share', st['alpha_share'], 'flat', st['flat'], 'top', st['top'])
