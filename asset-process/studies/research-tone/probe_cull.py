"""Does the client cull faces by winding? Compare the harness ID picture with ortho views: of the faces the harness
shows in front (gate camera), what share have facing > 0 toward that camera direction."""
import sys, pathlib; sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import numpy as np, collections
import tl, ortho
from tl import L
for npc in (7541, 11278):
    m, kv = tl.real_mesh(npc); a = tl.arrays(m); d = tl.real_def(npc); seqs = L.sequences(npc); cam = tl.camera(npc)
    ids = [tl.WORK / 'ids' / f'real-{npc}-id{k}.json' for k in (0, 1)]
    for yaw in (192, 1216):
        cells, _ = tl.draw([tl.entry(p, seqs) for p in ids], ['bind'], cam, yaw); i0, i1 = cells[0]
        c0, c1 = tl.decode_codes(i0), tl.decode_codes(i1); f = (c0 + tl.NCODE * c1)[(c0 >= 0) & (c1 >= 0)]
        cnt = collections.Counter(f.tolist())
        # camera direction: yaw in 2048ths about the vertical, pitch 128/2048 of a turn above the horizon; try both yaw senses
        for sgn in (1, -1):
            for off in (0, 1024):
                ang = sgn * (yaw + off) * 2 * np.pi / 2048; pit = 128 * 2 * np.pi / 2048
                c = np.array([np.sin(ang) * np.cos(pit), -np.sin(pit), np.cos(ang) * np.cos(pit)])
                fc = ortho.facing(a['V'], a['F'], c)
                px = sum(n for k, n in cnt.items()); pos = sum(n for k, n in cnt.items() if fc[k] > 0)
                print(npc, yaw, 'sense', sgn, 'offset', off, f'visible pixels on faces with facing>0: {pos / px:.3f}')
    print('depth offsets', collections.Counter(a['D'].tolist()), 'alpha', collections.Counter(a['A'].tolist()).most_common(8))
